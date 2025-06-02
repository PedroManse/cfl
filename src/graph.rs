use std::collections::HashMap;
use std::string::FromUtf8Error;
use crate::ChunkId;
use super::*;


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
            PieceContent::PStr(s)=>PieceKey::PStr(s),
            PieceContent::PInt(s)=>PieceKey::PInt(s),
            PieceContent::PUint(s)=>PieceKey::PUint(s),
            PieceContent::PMap(_) => return Err(ParseContentError::PieceCantBeKey { id: self.id, tag: self.tag }),
            PieceContent::PArray(_) => return Err(ParseContentError::PieceCantBeKey { id: self.id, tag: self.tag }),
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

impl PieceManager {
    //TODO get chunks
    pub fn new(chunks: Vec<Chunk>) -> Self {
        let chunks = chunks.into_iter().map(|c| (c.id, c)).collect();
        Self { chunks }
    }
    pub fn eval_first(&mut self) -> Result<Piece, ParseContentError> {
        self.eval_chunk_id(ChunkId(1))
    }
    pub fn eval_chunk_id(&mut self, chunk_id: ChunkId) -> Result<Piece, ParseContentError> {
        let x = self.chunks.remove(&chunk_id).ok_or(ParseContentError::ChunkNotFound(chunk_id))?;
        self.eval_chunk(x)
    }
    pub fn eval_chunk(&mut self, Chunk { tag, id, content, size: _ }: Chunk) -> Result<Piece, ParseContentError> {
        let pcontent = match tag {
            Tag::Int => {
                let v = make_int(&content);
                PieceContent::PInt(v)
            }
            Tag::Uint => {
                let v = make_uint(&content);
                PieceContent::PUint(v)
            }
            Tag::String => {
                let v = String::try_from(content)?;
                PieceContent::PStr(v)
            }
            Tag::Array => {
                // TODO use .array_chunks when stable
                let v: Vec<ChunkId> = content.chunks_exact(2).map(|v|{
                    let h = v[0];
                    let l = v[1];
                    ((h as u16) << 8) + (l as u16)
                }).map(ChunkId).collect();
                PieceContent::PArray(v)
            }
            Tag::Map => {
                let v: Result<Vec<(PieceKey, ChunkId)>, ParseContentError> = content.chunks_exact(4).map(|v|{
                    let hk = v[0];
                    let lk = v[1];
                    let hv = v[2];
                    let lv = v[3];
                    let k = ChunkId(((hk as u16) << 8) + (lk as u16));
                    let v = ChunkId(((hv as u16) << 8) + (lv as u16));
                    let k = self.eval_chunk_id(k)?.into_key()?;
                    Ok((k, v))
                }).collect();
                PieceContent::PMap(v?)
            }
        };
        Ok(Piece { id, tag, content: pcontent })
    }
}

fn make_int(content: &[u8]) -> i64 {
    content.iter().fold(0, |s, n| {
        println!("{} + {n}", s<<8);
        (s<<8)+(*n as i64)
    })
}

fn make_uint(content: &[u8]) -> u64 {
    content.iter().fold(0, |s, n| (s<<8)+(*n as u64))
}

#[derive(thiserror::Error, Debug)]
pub enum ParseContentError {
    #[error(transparent)]
    StringParse(#[from] FromUtf8Error),
    #[error("Chunk {0} not found")]
    ChunkNotFound(ChunkId),
    #[error("Chunk {id} can't be used as map key | It's a {tag:?}")]
    PieceCantBeKey{id: ChunkId, tag: Tag},
}

