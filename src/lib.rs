pub mod graph;
pub mod reader;
pub mod writer;
use std::fmt::Display;
use std::num::NonZeroU16;

// TODO should be enum of Invalid | NonZero<u16>
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ChunkId(pub NonZeroU16);

#[derive(Debug, Clone, Copy)]
pub struct ChunkSize(pub u16);

/// Chunk:
/// [ u16 Id | u8 Tag | u16 size | Content ]
#[derive(Debug, Clone)]
pub struct Chunk {
    pub id: ChunkId,
    pub tag: Tag,
    pub size: ChunkSize,
    pub content: Vec<u8>,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Tag {
    Int = 1,
    Uint = 2,
    String = 3,
    Array = 4,
    Map = 5,
}

impl Tag {
    pub fn check_valid_size(&self, size: ChunkSize) -> Option<reader::ReaderError> {
        use Tag::*;
        use reader::ReaderError as TSE;
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
pub struct Error {
    kind: reader::ReaderError,
    chunk_id: Option<ChunkId>,
    byte: usize,
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
        if let Some(ch_id) = self.chunk_id {
            write!(f, "Chunk {} | ", ch_id)?;
        } else {
            write!(f, "Byte @{} |", self.byte)?;
        }
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
        write!(f, "#{self}")
    }
}
