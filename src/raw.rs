use std::collections::HashSet;
use std::fmt::Display;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ChunkId(pub u16);
#[derive(Debug, Clone, Copy)]
pub struct ChunkSize(pub u16);

pub struct ChunkReader {
    ids_read: HashSet<ChunkId>
}

#[derive(Debug)]
pub struct Chunk {
    tag: Tag,
    id: ChunkId,
    size: ChunkSize,
    content: Box<[u8]>,
}

impl ChunkReader {
    pub fn new() -> Self {
        Self{ids_read: HashSet::new()}
    }
    pub fn read(&mut self, tag: Tag, id: ChunkId, size: ChunkSize, content: Box<[u8]>) -> Result<Chunk, Error> {
        if size.0 as usize != content.len() {
            return Err(ErrorKind::UnmatchedContentLen {
                said_size: size,
                actual_size: content.len(),
            }
            .into_error(id));
        }
        if !self.ids_read.insert(id) {
            return Err(ErrorKind::IdColision { id }.into_error(id));
        }
        if let Some(kind) = tag.check_valid_size(size) {
            return Err(ErrorKind::InvalidSizeForTag(kind).into_error(id));
        }

        Ok(Chunk {
            size,
            tag,
            id,
            content,
        })
    }
}

#[derive(Debug)]
pub enum Tag {
    Int = 1,
    Uint = 2,
    String = 3,
    Array = 4,
    Map = 5,
}

impl Tag {
    fn check_valid_size(&self, size: ChunkSize) -> Option<TagSizeError> {
        use Tag::*;
        use TagSizeError as TSE;
        match (self, size.0) {
            (Int, n) if n.count_ones() == 1 => None,
            (Int, _) => Some(TSE::IntMustBePowerOfTwo(size)),
            (Uint, n) if n.count_ones() == 1 => None,
            (Uint, _) => Some(TSE::UintMustBePowerOfTwo(size)),
            (String, _) => None,
            (Array, count) if (count & 0b1) == 0 => None,
            (Array, _) => Some(TSE::ArrayWithOddCount(size)),
            (Map, count) if (count & 0b11) == 0 => None,
            (Map, _) => Some(TSE::MapWithNonQuadCount(size)),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TagSizeError {
    #[error("Int chunk's size must be a power of two, got {0}")]
    IntMustBePowerOfTwo(ChunkSize),
    #[error("Uint chunk's size must be a power of two, got {0}")]
    UintMustBePowerOfTwo(ChunkSize),
    #[error("Array chunk size must be an even value, got {0}")]
    ArrayWithOddCount(ChunkSize),
    #[error("Map chunk size must be divisible by four, got {0}")]
    MapWithNonQuadCount(ChunkSize),
}

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error(transparent)]
    InvalidSizeForTag(TagSizeError),
    #[error("Advertised and actual size don't match")]
    UnmatchedContentLen {
        said_size: ChunkSize,
        actual_size: usize,
    },
    #[error("Chunk's id colision: Id {id:?} already taken")]
    IdColision { id: ChunkId },
}

impl ErrorKind {
    fn into_error(self, chunk_id: ChunkId) -> Error {
        Error {
            kind: self,
            chunk_id,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub struct Error {
    kind: ErrorKind,
    chunk_id: ChunkId,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Chunk #{} | ", self.chunk_id.0)?;
        write!(f, "{}", self.kind)
    }
}

impl Display for ChunkSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} Bytes", self.0)
    }
}

impl Display for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}
