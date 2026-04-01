use super::*;
use std::collections::HashSet;

pub struct ChunkReader<'b> {
    reader: ByteReader<'b>,
    ids_read: HashSet<ChunkId>,
}

pub struct ChunkIter<'b>(ChunkReader<'b>);

impl<'b> Iterator for ChunkIter<'b> {
    type Item = Result<Chunk, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.get_next_chunk()
    }
}

impl<'b> IntoIterator for ChunkReader<'b> {
    type Item = Result<Chunk, Error>;
    type IntoIter = ChunkIter<'b>;
    fn into_iter(self) -> Self::IntoIter {
        ChunkIter(self)
    }
}

impl<'b> ChunkReader<'b> {
    pub fn new(content: &'b [u8]) -> Self {
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
            .and_then(ChunkId::try_from)
            .map_err(|e| e.with_byte(self.reader.counter))?;
        let ChunkInfo { tag, size, content } = self
            .get_chunk_info()
            .map_err(|e| e.into_error(id, self.reader.counter))?;
        self.parse(tag, id, size, content)
            .map_err(|e| e.into_error(id, self.reader.counter))
    }

    fn parse(
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
            return Err(kind);
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

struct ByteReader<'b> {
    bytes: &'b [u8],
    counter: usize,
}

impl<'b> ByteReader<'b> {
    fn new(bytes: &'b [u8]) -> ByteReader<'b> {
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
    #[error("Int chunk's size must be a power of two, got {0}")]
    IntMustBePowerOfTwo(ChunkSize),
    #[error("Uint chunk's size must be a power of two, got {0}")]
    UintMustBePowerOfTwo(ChunkSize),
    #[error("Array chunk size must be an even value, got {0}")]
    ArrayWithOddCount(ChunkSize),
    #[error("Map chunk size must be divisible by four, got {0}")]
    MapWithNonQuadCount(ChunkSize),
    #[error("Chunk's id is 0")]
    InvalidChunk,
}

impl ReaderError {
    fn with_byte(self, byte: usize) -> Error {
        Error {
            kind: self.into(),
            chunk_id: None,
            byte,
        }
    }
    fn into_error(self, chunk_id: ChunkId, byte: usize) -> Error {
        Error {
            kind: self,
            chunk_id: Some(chunk_id),
            byte,
        }
    }
}

impl ChunkId {
    pub const fn try_from_u16(v: u16) -> Result<Self, ReaderError> {
        match NonZeroU16::new(v) {
            Some(v) => Ok(ChunkId(v)),
            None => Err(ReaderError::InvalidChunk),
        }
    }
    pub const unsafe fn from_u16_unchecked(v: u16) -> ChunkId {
        ChunkId(unsafe { NonZeroU16::new_unchecked(v) })
    }
}

impl TryFrom<u16> for ChunkId {
    type Error = reader::ReaderError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        NonZeroU16::new(value)
            .map(ChunkId)
            .ok_or(ReaderError::InvalidChunk)
    }
}
