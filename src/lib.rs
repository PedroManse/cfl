pub mod reader;
use std::fmt::Display;

// TODO should be enum of Invalid | NonZero<u16>
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ChunkId(pub u16);
#[derive(Debug, Clone, Copy)]
pub struct ChunkSize(pub u16);

#[derive(Debug)]
pub struct Chunk {
    pub id: ChunkId,
    pub tag: Tag,
    pub size: ChunkSize,
    pub content: Vec<u8>,
}

#[repr(u8)]
#[derive(Debug)]
pub enum Tag {
    EOF = 0,
    Int = 1,
    Uint = 2,
    String = 3,
    Array = 4,
    Map = 5,
}

impl Tag {
    pub fn check_valid_size(&self, size: ChunkSize) -> Option<TagSizeError> {
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
            (EOF, 0) => None,
            (EOF, _) => Some(TSE::EOFWithSize(size)),
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
    #[error("EOF tag must have size of 0, got {0}")]
    EOFWithSize(ChunkSize),
}


#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error(transparent)]
    InvalidChunk(#[from] reader::ReaderError),

}

impl ErrorKind {
    pub fn into_error(self, chunk_id: ChunkId) -> Error {
        Error {
            kind: self,
            chunk_id,
        }
    }
}

//TODO add where the error happened (what byte)
#[derive(thiserror::Error, Debug)]
pub struct Error {
    kind: ErrorKind,
    chunk_id: ChunkId,
}

// {tag}#{id}[{content}]
impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{}", self.tag, self.id)?;
        f.debug_list().entries(&self.content).finish()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Chunk {} | ", self.chunk_id.0)?;
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
        match  self.0 {
            0 => write!(f, "#InvalidId"),
            n => write!(f, "#{n}"),
        }
    }
}
