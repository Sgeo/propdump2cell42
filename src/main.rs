#[macro_use]
extern crate lazy_static;
extern crate libloading as lib;
extern crate byteorder;

pub mod ctree;

fn main() -> Result<(), Box<std::error::Error>> {
    ctree::init()?;
    ctree::DatFile::open("cell.idx")?;
    Ok(())
}
