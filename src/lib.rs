pub mod raw;
use std::fmt::Display;

use self::raw::Chunk;

// TODO should be enum of Invalid | NonZero<u16>
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ChunkId(pub u16);
#[derive(Debug, Clone, Copy)]
pub struct ChunkSize(pub u16);

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error(transparent)]
    InvalidChunk(#[from] raw::reader::ReaderError),

}

impl ErrorKind {
    pub fn into_error(self, chunk_id: ChunkId) -> Error {
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
