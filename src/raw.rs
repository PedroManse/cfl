use super::*;
pub mod reader;

#[derive(Debug)]
pub struct Chunk {
    pub tag: Tag,
    pub id: ChunkId,
    pub size: ChunkSize,
    pub content: Vec<u8>,
}

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

