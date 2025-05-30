use cfl::{Error, Tag, reader::ChunkReader};

fn exec() -> Result<(), Error> {
    let cont = vec![0, 1, Tag::String as u8, 0, 4, 104, 105, 33, 10];
    let reader = ChunkReader::new(cont).into_iter();
    let ch: Vec<_> = reader.collect::<Result<_, _>>()?;
    for c in ch {
        println!("{c}");
    }
    Ok(())
}

fn main() {
    if let Err(e) = exec() {
        eprintln!("{e}")
    }
}
