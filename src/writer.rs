use std::io::Write;

use crate::Chunk;
pub struct ChunkWriter<'w, W: Write>(pub &'w mut W);

impl<'w, W: Write> ChunkWriter<'w, W> {
    pub fn write_chunks(&mut self, chunks: &[Chunk]) -> Result<(), std::io::Error> {
        for chunk in chunks {
            self.write_chunk(chunk)?;
        }
        Ok(())
    }
    pub fn write_chunk(&mut self, chunk: &Chunk) -> Result<(), std::io::Error> {
        let id = chunk.id.0.get().to_be_bytes();
        let tag = (chunk.tag) as u8;
        let size = chunk.size.0.to_be_bytes();
        self.0.write_all(&id)?;
        self.0.write_all(&[tag])?;
        self.0.write_all(&size)?;
        self.0.write_all(&chunk.content)?;

        Ok(())
    }
}
