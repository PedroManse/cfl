use cfl::{
    Tag, Error,
    graph::PieceManager,
    reader::ChunkReader,
};

fn exec() -> Result<(), Error> {
    let cont = vec![0, 1, Tag::String as u8, 0, 4, 104, 105, 33, 10];
    let reader = ChunkReader::new(cont).into_iter();
    let ch: Vec<_> = reader.collect::<Result<_, _>>()?;
    let mut graph_rez = PieceManager::new(ch);
    let x = graph_rez.eval_chunk_id(cfl::ChunkId(1));
    println!("{x:?}");
    Ok(())
}

fn main() {
    if let Err(e) = exec() {
        eprintln!("{e}")
    }
}
