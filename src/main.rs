use cfl::{
    graph::PieceManager, raw::{
        reader::ChunkReader, Tag
    }, Error
};

fn exec() -> Result<(), Error> {
    let mut itr = vec![0, 1, Tag::Uint as u8, 0, 4, 0, 1, 0, 0].into_iter();
    let mut reader = ChunkReader::new();
    let ch = reader.get_chunk(&mut itr)?;

    let mut graph_rez = PieceManager::new();
    let ch = graph_rez.eval_chunk(ch).unwrap();
    println!("{ch:?}");

    Ok(())
}

fn main() {
    if let Err(e) = exec() {
        eprint!("{e}")
    }
}
