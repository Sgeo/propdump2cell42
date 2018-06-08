#[macro_use]
extern crate lazy_static;
extern crate libloading as lib;
extern crate byteorder;
#[macro_use] extern crate failure;

use byteorder::{ByteOrder, LE};

mod ctree;
mod aw;
mod propdump;

#[derive(Debug)]
/// Cannot (yet) rewrite already written cells
struct ObjectWriter<'idx, 'dat> {
    cell: Option<(i16, i16)>,
    idx: &'idx ctree::IdxFile,
    dat: &'dat ctree::DatFile,
    cell_data_buffer: Vec<u8>
}

impl<'idx, 'dat> ObjectWriter<'idx, 'dat> {
    pub fn new(idx: &'idx ctree::IdxFile, dat: &'dat ctree::DatFile) -> Self {
        ObjectWriter {
            cell: None,
            idx: idx,
            dat: dat,
            cell_data_buffer: vec![]
        }
    }

    pub fn add_object(&mut self, object: &aw::Object) -> Result<(), failure::Error> {
        let loc = object.location();
        if self.cell.is_some() && self.cell != Some((loc.cell_x, loc.cell_z)) {
            self.write_current_cell()?;
        }
        self.cell = Some((loc.cell_x, loc.cell_z));
        object.write(&mut self.cell_data_buffer)?;
        Ok(())
    }
    
    pub fn write_current_cell(&mut self) -> Result<(), failure::Error> {
        if self.cell.is_none() {
            return Ok(());
        }
        let (cell_x, cell_z) = self.cell.unwrap();
        self.cell = None;
        // Currently hard to avoid accidental appending to cell sequence, and it seems to be unneeded for AW
        // let mut sequence_key = [0u8; 6];
        // let mut sequence_value = [0u8; 4];
        // LE::write_u16(&mut sequence_key[0..2], 0);
        // LE::write_i16(&mut sequence_key[2..4], cell_x);
        // LE::write_i16(&mut sequence_key[4..6], cell_z);
        // LE::write_i32(&mut sequence_value, 1);
        // ctree::insert(&self.idx, &self.dat, &sequence_key, &sequence_value)?;
        let mut celldata_key = [0u8; 6];
        LE::write_u16(&mut celldata_key[0..2], 1);
        LE::write_i16(&mut celldata_key[2..4], cell_x);
        LE::write_i16(&mut celldata_key[4..6], cell_z);
        ctree::insert_or_append(&self.idx, &self.dat, &celldata_key, &self.cell_data_buffer)?;
        self.cell_data_buffer.clear();
        Ok(())
    }
}

impl<'idx, 'dat> Drop for ObjectWriter<'idx, 'dat> {
    fn drop(&mut self) {
        self.write_current_cell().unwrap();
    }
}

fn main() -> Result<(), failure::Error> {
    use std::fs;
    use std::io;
    use std::env;
    fs::copy("blank42.dat", "cell.dat")?;
    fs::copy("blank42.idx", "cell.idx")?;
    ctree::init()?;
    let dat = ctree::DatFile::open("cell.dat")?;
    let idx = ctree::IdxFile::open("cell.idx")?;
    let filename = match env::args().nth(1) {
        Some(filename) => filename,
        None => bail!("Please provide a filename!")
    };
    let propdump_file = io::BufReader::new(fs::File::open(filename)?);
    let propdump = propdump::Propdump::new(propdump_file)?;
    let mut writer = ObjectWriter::new(&idx, &dat);
    for object in propdump {
        println!("{:?}", object.location());
        writer.add_object(&object)?;
    }
    
    Ok(())
}
