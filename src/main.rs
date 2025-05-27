use cfl::{
    Error,
    raw::{
        Tag,
        reader::ChunkReader,
    },
};

fn exec() -> Result<(), Error> {
    let mut itr = vec![0, Tag::Int as u8, 0, 1, 0, 4, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0].into_iter();
    let mut reader = ChunkReader::new();
    let ch = reader.get_chunk(&mut itr)?;
    let cx = reader.get_chunk(&mut itr)?;

    println!("{ch}\n{cx}");
    Ok(())
}

fn main() {
    if let Err(e) = exec() {
        eprint!("{e}")
    }
}
