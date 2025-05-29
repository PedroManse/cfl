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

    pub fn get_chunk<I: Iterator<Item = u8>>(&mut self, itr: &mut I) -> Result<Chunk, Error> {
        use ReaderError::*;
        let id = read_u16_chunked_err(itr, EOF, MissingId)
            .map_err(ErrorKind::from)
            .map_err(|e| e.into_error(ChunkId(0)))
            .map(ChunkId)?;
        let make_err = |m: Option<u16>, e| {
            m.ok_or(e)
                .map_err(ErrorKind::from)
                .map_err(|e| e.into_error(id))
        };
        let tag = itr.next().ok_or(MissingTag)
                .map_err(ErrorKind::from)
                .map_err(|e| e.into_error(id))?;
        let size = make_err(read_u16(itr), MissingSize).map(ChunkSize)?;
        let content = get_n_u8s(itr, size.0)
            .map_err(|got|MissingContent{needs: size.0, got: got.len()})
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

fn get_n_u8s<I: Iterator<Item = u8>>(itr: &mut I, size: u16) -> Result<Vec<u8>, Vec<u8>> {
    let mut vec = Vec::with_capacity(size as usize);
    for _ in 0..size {
        match itr.next() {
            Some(v)=>vec.push(v),
            None => return Err(vec)
        };
    }
    Ok(vec)
}

fn read_u16_chunked_err<I: Iterator<Item = u8>, E>(itr: &mut I, m1: E, m2: E) -> Result<u16, E> {
    Ok(merge_2be_u8s(itr.next().ok_or(m1)?, itr.next().ok_or(m2)?))
}

fn read_u16<I: Iterator<Item = u8>>(itr: &mut I) -> Option<u16> {
    Some(merge_2be_u8s(itr.next()?, itr.next()?))
}

impl TryFrom<u8> for Tag {
    type Error = ReaderError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Tag::*;
        Ok(match value {
            0 => EOF,
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
    #[error("Tried to start reading chunk and reached EOF")]
    EOF,
    #[error("Chunk only has one byte for id, need two")]
    MissingId,
    #[error("Missing size")]
    MissingSize,
    #[error("Missing tag")]
    MissingTag,
    #[error("Missing Content, expected {needs} bytes, got {got}")]
    MissingContent{needs: u16, got: usize},
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
    InvalidSizeForTag(TagSizeError),
}
