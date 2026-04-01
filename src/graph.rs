use super::*;
use crate::ChunkId;
use std::collections::HashMap;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub struct Piece {
    id: ChunkId,
    tag: Tag,
    content: PieceContent,
}

#[derive(Debug)]
pub enum PieceContent {
    PStr(String),
    PInt(i64),
    PUint(u64),
    PArray(Vec<ChunkId>),
    PMap(Vec<(PieceKey, ChunkId)>),
}

impl Piece {
    fn into_key(self) -> Result<PieceKey, ParseContentError> {
        Ok(match self.content {
            PieceContent::PStr(s) => PieceKey::PStr(s),
            PieceContent::PInt(s) => PieceKey::PInt(s),
            PieceContent::PUint(s) => PieceKey::PUint(s),
            PieceContent::PMap(_) => {
                return Err(ParseContentError::PieceCantBeKey {
                    id: self.id,
                    tag: self.tag,
                });
            }
            PieceContent::PArray(_) => {
                return Err(ParseContentError::PieceCantBeKey {
                    id: self.id,
                    tag: self.tag,
                });
            }
        })
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum PieceKey {
    PStr(String),
    PInt(i64),
    PUint(u64),
}

#[derive(Default)]
pub struct PieceManager {
    chunks: HashMap<ChunkId, Chunk>,
}

pub struct Consume;
pub struct Eval;

pub trait ChunkGraphReadType<PM> {
    fn read_first(pm: PM) -> Result<Piece, ParseContentError> {
        // SAFETY: [NonZeroU16::new_unchecked] is safe in const contexts, therefore
        // [ChunkId::from_u16_unchecked] is too.
        const FIRST_CHUNK_ID: ChunkId = unsafe { ChunkId::from_u16_unchecked(1) };
        Self::read_chunk_id(pm, FIRST_CHUNK_ID)
    }
    fn read_chunk_and_id(pm: PM, chunk_id: u16) -> Result<Piece, ParseContentError> {
        let id = ChunkId::try_from(chunk_id)?;
        Self::read_chunk_id(pm, id)
    }
    fn read_chunk_id(pm: PM, chunk_id: ChunkId) -> Result<Piece, ParseContentError>;
}

trait ChunkGraphResolver<PM>: ChunkGraphReadType<PM> {
    fn _read_chunk(
        pm: PM,
        Chunk {
            id, tag, content, ..
        }: Chunk,
    ) -> Result<Piece, ParseContentError> {
        let pcontent = match tag {
            Tag::Int => {
                let v = content.iter().fold(0, |s, n| {
                    println!("{} + {n}", s << 8);
                    (s << 8) + (*n as i64)
                });
                PieceContent::PInt(v)
            }
            Tag::Uint => {
                let v = content.iter().fold(0, |s, n| (s << 8) + (*n as u64));
                PieceContent::PUint(v)
            }
            Tag::String => {
                let v = String::try_from(content.to_owned())?;
                PieceContent::PStr(v)
            }
            Tag::Array => {
                let v = content
                    .as_chunks()
                    .0
                    .iter()
                    .map(|&[h, l]| ((h as u16) << 8) + (l as u16))
                    .map(|i| ChunkId::try_from(i))
                    .collect::<Result<_, _>>()?;
                PieceContent::PArray(v)
            }
            Tag::Map => Self::_read_map_chunk(pm, content)?,
        };
        Ok(Piece {
            id,
            tag,
            content: pcontent,
        })
    }
    fn _read_map_chunk(pm: PM, content: Vec<u8>) -> Result<PieceContent, ParseContentError>;
}

impl ChunkGraphResolver<&mut PieceManager> for Consume {
    fn _read_map_chunk(
        pm: &mut PieceManager,
        content: Vec<u8>,
    ) -> Result<PieceContent, ParseContentError> {
        let v: Result<Vec<(PieceKey, ChunkId)>, ParseContentError> = content
            .as_chunks()
            .0
            .iter()
            .map(|&[hk, lk, hv, lv]| {
                let k = ChunkId::try_from(((hk as u16) << 8) + (lk as u16))?;
                let v = ChunkId::try_from(((hv as u16) << 8) + (lv as u16))?;
                let k = Self::read_chunk_id(pm, k)?.into_key()?;
                Ok((k, v))
            })
            .collect();
        v.map(PieceContent::PMap)
    }
}

impl ChunkGraphReadType<&mut PieceManager> for Consume {
    fn read_chunk_id(pm: &mut PieceManager, chunk_id: ChunkId) -> Result<Piece, ParseContentError> {
        let x = pm
            .chunks
            .remove(&chunk_id)
            .ok_or(ParseContentError::ChunkNotFound(chunk_id))?;
        Self::_read_chunk(pm, x)
    }
}

impl ChunkGraphResolver<&PieceManager> for Eval {
    fn _read_map_chunk(
        pm: &PieceManager,
        content: Vec<u8>,
    ) -> Result<PieceContent, ParseContentError> {
        let v: Result<Vec<(PieceKey, ChunkId)>, ParseContentError> = content
            .as_chunks()
            .0
            .iter()
            .map(|&[hk, lk, hv, lv]| {
                let k = ChunkId::try_from(((hk as u16) << 8) + (lk as u16))?;
                let v = ChunkId::try_from(((hv as u16) << 8) + (lv as u16))?;
                let k = Self::read_chunk_id(pm, k)?.into_key()?;
                Ok((k, v))
            })
            .collect();
        v.map(PieceContent::PMap)
    }
}

impl ChunkGraphReadType<&PieceManager> for Eval {
    fn read_chunk_id(pm: &PieceManager, chunk_id: ChunkId) -> Result<Piece, ParseContentError> {
        let x = pm
            .chunks
            .get(&chunk_id)
            .cloned()
            .ok_or(ParseContentError::ChunkNotFound(chunk_id))?;
        Self::_read_chunk(pm, x)
    }
}

impl PieceManager {
    pub fn new(chunks: Vec<Chunk>) -> Self {
        let chunks = chunks.into_iter().map(|c| (c.id, c)).collect();
        Self { chunks }
    }
    pub fn count_bytes(&self) -> usize {
        self.chunks
            .iter()
            .map(|(_, c)| 5 + usize::from(c.size.0))
            .sum()
    }
    pub fn into_chunks(self) -> Vec<Chunk> {
        self.chunks.into_values().collect()
    }
    pub fn into_inner(self) -> HashMap<ChunkId, Chunk> {
        self.chunks
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseContentError {
    #[error(transparent)]
    ReaderError(#[from] reader::ReaderError),
    #[error(transparent)]
    StringParse(#[from] FromUtf8Error),
    #[error("Chunk {0} not found")]
    ChunkNotFound(ChunkId),
    #[error("Chunk {id} can't be used as map key | It's a {tag:?}")]
    PieceCantBeKey { id: ChunkId, tag: Tag },
}
