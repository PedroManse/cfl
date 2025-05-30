use super::*;
use std::collections::HashSet;

pub struct ChunkReader {
    reader: ByteReader,
    ids_read: HashSet<ChunkId>,
}

pub struct ChunkIter ( ChunkReader );

impl Iterator for ChunkIter {
    type Item = Result<Chunk, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.get_next_chunk()
    }
}

impl IntoIterator for ChunkReader {
    type Item = Result<Chunk, Error>;
    type IntoIter = ChunkIter;
    fn into_iter(self) -> Self::IntoIter {
        ChunkIter(self)
    }
}

impl ChunkReader {
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            reader: ByteReader::new(content),
            ids_read: HashSet::new(),
        }
    }

    pub fn get_next_chunk(&mut self) -> Option<Result<Chunk, Error>> {
        if self.reader.has_more() {
            Some(self.get_chunk())
        } else {
            None
        }
    }

    pub fn get_chunk(&mut self) -> Result<Chunk, Error> {
        let id = self
            .reader
            .u16_chunked_err(ReaderError::EOF, ReaderError::MissingId)
            .map(ChunkId)
            .map_err(|e| e.into_error(ChunkId(0)))?;
        let ChunkInfo { tag, size, content } =
            self.get_chunk_info().map_err(|e| e.into_error(id))?;
        self.read(tag, id, size, content)
            .map_err(|e| e.into_error(id))
    }

    fn read(
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

    fn get_chunk_info(&mut self) -> Result<ChunkInfo, ReaderError> {
        use ReaderError::*;
        let tag = self
            .reader
            .u8()
            .ok_or(MissingTag)
            .and_then(TryInto::<Tag>::try_into)?;
        let size = self.reader.u16().map(ChunkSize).ok_or(MissingSize)?;
        let content = self
            .reader
            .get_n_u8s(size.0)
            .map_err(|got| MissingContent {
                needs: size.0,
                got: got.len(),
            })?;
        Ok(ChunkInfo { tag, size, content })
    }
}

struct ChunkInfo {
    tag: Tag,
    size: ChunkSize,
    content: Vec<u8>,
}

struct ByteReader {
    bytes: Vec<u8>,
    counter: usize,
}

impl ByteReader {
    fn new(bytes: Vec<u8>) -> Self {
        Self { bytes, counter: 0 }
    }

    fn has_more(&self) -> bool {
        self.counter < self.bytes.len()
    }

    fn u8(&mut self) -> Option<u8> {
        let v = self.bytes.get(self.counter)?;
        self.counter += 1;
        Some(*v)
    }

    fn u16(&mut self) -> Option<u16> {
        Some((self.u8()? as u16) << 8 | self.u8()? as u16)
    }

    fn u16_chunked_err<E>(&mut self, m1: E, m2: E) -> Result<u16, E> {
        Ok(merge_2be_u8s(self.u8().ok_or(m1)?, self.u8().ok_or(m2)?))
    }

    fn get_n_u8s(&mut self, size: u16) -> Result<Vec<u8>, Vec<u8>> {
        let mut vec = Vec::with_capacity(size as usize);
        for _ in 0..size {
            match self.u8() {
                Some(v) => vec.push(v),
                None => return Err(vec),
            };
        }
        Ok(vec)
    }
}

fn merge_2be_u8s(high: u8, low: u8) -> u16 {
    (high as u16) << 8 | low as u16
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
    MissingContent { needs: u16, got: usize },
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
