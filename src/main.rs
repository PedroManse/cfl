use std::num::NonZero;

use cfl::graph::ChunkGraphReadType;
use cfl::{Error, graph::PieceManager};

fn exec() -> Result<(), Error> {
    let ch = vec![cfl::Chunk {
        id: cfl::ChunkId(NonZero::<u16>::new(1).unwrap()),
        tag: cfl::Tag::String,
        size: cfl::ChunkSize(5),
        content: "hello".as_bytes().to_vec(),
    }];
    println!("chunks: {ch:?}");
    let graph_rez = PieceManager::new(ch);
    let x = cfl::graph::Eval::read_first(&graph_rez);
    println!("graph: {x:?}");

    let mut out = Vec::with_capacity(graph_rez.count_bytes());
    cfl::writer::ChunkWriter(&mut out)
        .write_chunks(&graph_rez.into_chunks())
        .unwrap();
    println!("byte out:{out:?}");

    Ok(())
}

fn main() {
    if let Err(e) = exec() {
        eprintln!("{e}")
    }
}
