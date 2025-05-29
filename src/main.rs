use cfl::{
    Error,
    reader::ChunkReader,
    Tag
};

fn exec() -> Result<(), Error> {
    let mut itr = vec![
        0, 1,
        0, Tag::Uint as u8,
        0, 4,
        10, 0, 0, 100].into_iter();
    let mut reader = ChunkReader::new();
    let ch = reader.get_chunk(&mut itr)?;
    println!("{ch}");
    Ok(())
}

fn main() {
    if let Err(e) = exec() {
        eprintln!("{e}")
    }
}
