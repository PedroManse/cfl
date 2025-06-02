use super::*;
use std::collections::HashSet;

pub struct ChunkReader {
    ids_read: HashSet<ChunkId>,
}

impl ChunkReader {
    pub fn new() -> Self {
        Self {
            ids_read: HashSet::new(),
        }
    }
    pub fn read(
        &mut self,
        tag: Tag,
        id: ChunkId,
        size: ChunkSize,
        content: Vec<u8>,
    ) -> Result<Chunk, ReaderError> {
        if size.0 as usize != content.len() {
            return Err(ReaderError::UnmatchedContentLen {
                said_size: size,
                actual_size: content.len(),
            });
        }
        if !self.ids_read.insert(id) {
            return Err(ReaderError::IdColision { id });
        }
        if let Some(kind) = tag.check_valid_size(size) {
            return Err(ReaderError::InvalidSizeForTag(kind));
        }

        Ok(Chunk {
            size,
            tag,
            id,
            content,
        })
    }

    fn wrap_err<T>(m: Option<T>, e: ReaderError, id: ChunkId) -> Result<T, Error> {
        m.ok_or(e)
            .map_err(ErrorKind::from)
            .map_err(|e| e.into_error(id))
    }

    pub fn get_chunk<I: Iterator<Item = u8>>(&mut self, itr: &mut I) -> Result<Chunk, Error> {
        use ReaderError::*;
        let id = read_u16(itr)
            .ok_or(MissingId)
            .map_err(ErrorKind::from)
            .map_err(|e| e.into_error(ChunkId(u16::MAX)))
            .map(ChunkId)?;
        let tag = ChunkReader::wrap_err(itr.next(), MissingTag, id)?;
        let size = ChunkSize(ChunkReader::wrap_err(read_u16(itr), MissingTag, id)?);
        let content = get_n_u8s(itr, size.0)
            .ok_or(MissingContent)
            .map_err(ErrorKind::from)
            .map_err(|e| e.into_error(id))?;
        let tag = Tag::try_from(tag)
            .map_err(ErrorKind::from)
            .map_err(|e| e.into_error(id))?;
        self.read(tag, id, size, content)
            .map_err(ErrorKind::from)
            .map_err(|e| e.into_error(id))
    }
}

fn merge_2be_u8s(high: u8, low: u8) -> u16 {
    (high as u16) << 8 | low as u16
}

fn get_n_u8s<I: Iterator<Item = u8>>(itr: &mut I, size: u16) -> Option<Vec<u8>> {
    let mut vec = Vec::with_capacity(size as usize);
    for _ in 0..size {
        vec.push(itr.next()?);
    }
    Some(vec)
}

fn read_u16<I: Iterator<Item = u8>>(itr: &mut I) -> Option<u16> {
    Some(merge_2be_u8s(itr.next()?, itr.next()?))
}

impl TryFrom<u8> for Tag {
    type Error = ReaderError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Tag::*;
        Ok(match value {
            1 => Int,
            2 => Uint,
            3 => String,
            4 => Array,
            5 => Map,
            n => return Err(Self::Error::InvalidTagValue(n)),
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ReaderError {
    #[error("Missing ID")]
    MissingId,
    #[error("Missing size")]
    MissingSize,
    #[error("Missing tag")]
    MissingTag,
    #[error("Missing Content")]
    MissingContent,
    #[error("Invalid tag value: {0}")]
    InvalidTagValue(u8),
    #[error("Advertised and actual size don't match")]
    UnmatchedContentLen {
        said_size: ChunkSize,
        actual_size: usize,
    },
    #[error("Chunk's id colision: {id} already taken")]
    IdColision { id: ChunkId },
    #[error(transparent)]
    InvalidSizeForTag(raw::TagSizeError),
}
