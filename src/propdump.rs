use failure;

use std::io::{self, Read, BufRead};
use std::error;
use std::str::FromStr;

use aw::Object;

trait BufReadHelpers: BufRead {
    fn read_until_exclusive(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize>;
    fn read_item<N>(&mut self) -> Result<N, failure::Error>
        where N: FromStr,
              N::Err: error::Error + Send + Sync + 'static;
}

impl<R: BufRead> BufReadHelpers for R {
    fn read_until_exclusive(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        let result = self.read_until(byte, buf);
        buf.pop();
        result
    }
    
    fn read_item<N>(&mut self) -> Result<N, failure::Error>
        where N: FromStr,
              N::Err: error::Error + Send + Sync + 'static {
        let mut buf: Vec<u8> = Vec::new();
        self.read_until_exclusive(b' ', &mut buf)?;
        let s = ::std::str::from_utf8(&buf)?;
        Ok(N::from_str(s)?)
    }
}

#[derive(Debug)]
pub struct Propdump<R: BufRead> {
    file: R,
    v4: bool
}

impl<R: BufRead> Propdump<R> {
    pub fn new(mut file: R) -> Result<Self, failure::Error> {
        let mut first_line = String::new();
        file.read_line(&mut first_line)?;
        if first_line != "propdump version 3\r\n" && first_line != "propdump version 4\r\n" {
            bail!("Unrecognized first line of propdump!");
        }
        let v4 = first_line == "propdump version 4\r\n";
        Ok(Propdump {
            v4: v4,
            file: file
        })
    }
}

impl<R: BufRead> Iterator for Propdump<R> {
    type Item = Object;
    
    fn next(&mut self) -> Option<Self::Item> {
        let mut object = Object::default();
        let maybe_citnum = self.file.read_item();
        if let Err(err) = maybe_citnum {
            return None;
        }
        object.citnum = maybe_citnum.expect("Fatal error reading propdump");
        object.time = self.file.read_item().expect("Fatal error reading propdump");
        object.x = self.file.read_item().expect("Fatal error reading propdump");
        object.y = self.file.read_item().expect("Fatal error reading propdump");
        object.z = self.file.read_item().expect("Fatal error reading propdump");
        object.yaw = self.file.read_item().expect("Fatal error reading propdump");
        object.tilt = self.file.read_item().expect("Fatal error reading propdump");
        object.roll = self.file.read_item().expect("Fatal error reading propdump");
        if self.v4 {
            object.type_ = self.file.read_item().expect("Fatal error reading propdump");
        }
        let namelen: usize = self.file.read_item().expect("Fatal error reading propdump");
        let desclen: usize = self.file.read_item().expect("Fatal error reading propdump");
        println!("desclen: {}", desclen);
        let actionlen: usize = self.file.read_item().expect("Fatal error reading propdump");
        let datalen: usize = if self.v4 {
            self.file.read_item().expect("Fatal error reading propdump")
        } else {
            0
        };
        object.name = vec![0; namelen];
        object.desc = vec![0; desclen];
        println!("{:?}", object.desc);
        object.action = vec![0; actionlen];
        object.data = vec![0; datalen];
        self.file.read_exact(&mut object.name).expect("Fatal error reading propdump");
        let written = self.file.read_exact(&mut object.desc).expect("Fatal error reading propdump");
        println!("Len of slice: {}", (&mut object.desc).len());
        self.file.read_exact(&mut object.action).expect("Fatal error reading propdump");
        self.file.read_exact(&mut object.data).expect("Fatal error reading propdump");
        let _nl = self.file.read_exact(&mut [0u8; 2]); // Read past newline
        Some(object)
    }
}