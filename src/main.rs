use cfl::raw::{ ChunkId, ChunkReader, ChunkSize, Error, Tag };

fn do_test() {
    let example_info: Box<[u8]> = Box::new([
        0, Tag::Int as u8,
        0, 1,
        0, 4,
        0, 0, 0, 100
    ]);
    test(example_info);
}

fn merge_2be_u8s(high: u8, low: u8) -> u16 {
    (high as u16) << 8 | low as u16
}

fn get_n_u8s<I: Iterator<Item = u8>>(itr: &mut I, size: u16) -> Option<Vec<u8>> {
    let mut vec = Vec::with_capacity(size as usize);
    for _ in 0..size {
        vec.push(itr.next()?);
    }
    Some(vec)
}


fn test(info: Box<[u8]>) -> Option<()> {
    let mut info = info.into_iter();
    let tag = merge_2be_u8s(info.next()?, info.next()?);
    let id = merge_2be_u8s(info.next()?, info.next()?);
    let size = merge_2be_u8s(info.next()?, info.next()?);
    let content = get_n_u8s(&mut info, size);
    Some(())
}

fn exec() -> Result<(), Error> {
    let mut reader = ChunkReader::new();
    let c = reader.read(
        Tag::Int,
        ChunkId(0),
        ChunkSize(4),
        Box::new([0, 0, 0, 0])
    )?;
    println!("{c:?}");
    Ok(())
}

fn main() {
    if let Err(e) = exec() {
        eprint!("{e}")
    }
}
