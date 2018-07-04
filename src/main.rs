#[macro_use]
extern crate lazy_static;
extern crate libloading as lib;
extern crate byteorder;
extern crate ctrlc;
extern crate clap;
extern crate encoding;
#[macro_use] extern crate failure;

use byteorder::{ByteOrder, LE};
use std::sync::atomic::{AtomicBool, Ordering};
use std::str::FromStr;
use clap::{App, Arg};

mod ctree;
mod aw;
mod propdump;
mod teleports;

use teleports::Teleports;

static RUNNING: AtomicBool = AtomicBool::new(true);

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
        let result = self.write_current_cell();
        if result.is_err() {
            println!("Unable to write final cell!");
        }
    }
}

struct Config {
    teleports: Option<Teleports>,
    citnums: Option<Vec<i32>>
}

fn config() -> Result<Config, failure::Error> {
    let matches = App::new("Propdump to Cell 4.2")
        .author("Sgeo <sgeoster@gmail.com>")
        .about("Converts a propdump to Active Worlds 4.2 standalone cache files")
        .arg(Arg::with_name("teleports")
             .long("teleports")
             .short("t")
             .takes_value(true)
             .value_name("TELEPORTS")
             .requires("radius")
             .help("Specifies a teleport.txt file. Only data in an area near a point in the teleport.txt file will be included"))
        .arg(Arg::with_name("radius")
             .long("radius")
             .short("r")
             .takes_value(true)
             .value_name("RADIUS")
             .requires("teleports")
             .help("Specify how many coordinates north, west, east, and south of each location in the teleports to include"))
        .arg(Arg::with_name("citnum")
             .long("citnum")
             .short("c")
             .takes_value(true)
             .value_name("CITNUMS")
             .help("Specify one or more citizen numbers to include. If one or more are specified, all other citizen numbers are excluded"))
         .get_matches();
    let mut config = Config {
        teleports: None,
        citnums: None
    };
    if let Some(teleport_file_name) = matches.value_of("teleports") {
        let radius: i16 = i16::from_str(matches.value_of("radius").unwrap())?;
        config.teleports = Some(Teleports::from_file(teleport_file_name, radius)?);
    }
    if let Some(citnums) = matches.values_of("citnum") {
        config.citnums = Some(citnums.map(i32::from_str).map(Result::unwrap).collect());
    }
    Ok(config)
}

fn main() -> Result<(), failure::Error> {
    use std::fs;
    use std::io;
    ctrlc::set_handler(move || {
        println!("Received Ctrl-C");
        RUNNING.store(false, Ordering::SeqCst);
    })?;
    let config = config()?;
    fs::copy("blank42.dat", "cell.dat")?;
    fs::copy("blank42.idx", "cell.idx")?;
    ctree::init()?;
    let dat = ctree::DatFile::open("cell.dat")?;
    let idx = ctree::IdxFile::open("cell.idx")?;
    let stdin = io::stdin();
    let propdump_file = stdin.lock();
    let propdump = propdump::Propdump::new(propdump_file)?.filter(|obj| {
        if let Some(ref teleports) = config.teleports {
            if !teleports.contains(obj) {
                return false;
            }
        }
        if let Some(ref citnums) = config.citnums {
            if !citnums.contains(&obj.citnum) {
                return false;
            }
        }
        true
    });
    let mut writer = ObjectWriter::new(&idx, &dat);
    for object in propdump {
        if !RUNNING.load(Ordering::SeqCst) {
            println!("Quitting due to Ctrl-C");
            break;
        }
        writer.add_object(&object)?;
    }
    
    Ok(())
}
