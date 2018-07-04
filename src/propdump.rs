use failure;

use std::io::{self, Read, BufRead};
use std::error;
use std::str::FromStr;

use encoding::{Encoding, DecoderTrap};
use encoding::types::EncodingRef;
use encoding::all::{UTF_8, WINDOWS_1252};

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

fn restore_newlines(buffer: &mut [u8]) {
    if buffer.len() >= 2 {
        for i in 0..(buffer.len()-1) {
            if buffer[i+1] == b'\x7F' {
                buffer[i+1] = b'\n';
                if buffer[i] == b'\x80' {
                    buffer[i] = b'\r';
                }
            }
        }
    }
    if buffer.len() > 0 && buffer[0] == b'\x7F' {
        buffer[0] = b'\n';
    }
}

pub struct Propdump<R: BufRead> {
    file: R,
    v4: bool,
    encoding: EncodingRef,
}

impl<R: BufRead> Propdump<R> {
    pub fn new(mut file: R) -> Result<Self, failure::Error> {
        let mut first_line = String::new();
        file.read_line(&mut first_line)?;
        if first_line != "propdump version 3\r\n" && first_line != "propdump version 4\r\n" && first_line != "propdump version 5\r\n" {
            bail!("Unrecognized first line of propdump!");
        }
        let v4 = first_line == "propdump version 4\r\n";
        let encoding = if first_line == "propdump version 5\r\n" {
            UTF_8 as EncodingRef
        } else {
            WINDOWS_1252 as EncodingRef
        };
        Ok(Propdump {
            v4: v4,
            file: file,
            encoding: encoding
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
        let actionlen: usize = self.file.read_item().expect("Fatal error reading propdump");
        let datalen: usize = if self.v4 {
            self.file.read_item().expect("Fatal error reading propdump")
        } else {
            0
        };
        let mut name = vec![0; namelen];
        let mut desc = vec![0; desclen];
        let mut action = vec![0; actionlen];
        let mut hexdata = vec![0; datalen * 2];
        self.file.read_exact(&mut name).expect("Fatal error reading propdump");
        object.name = self.encoding.decode(&name, DecoderTrap::Replace).expect("Fatal decode error reading propdump");
        self.file.read_exact(&mut desc).expect("Fatal error reading propdump");
        restore_newlines(&mut desc);
        object.desc = self.encoding.decode(&desc, DecoderTrap::Replace).expect("Fatal decode error reading propdump");
        self.file.read_exact(&mut action).expect("Fatal error reading propdump");
        restore_newlines(&mut action);
        object.action = self.encoding.decode(&action, DecoderTrap::Replace).expect("Fatal decode error reading propdump");
        self.file.read_exact(&mut hexdata).expect("Fatal error reading propdump");
        object.data = hexdata.chunks(2).map(String::from_utf8_lossy).map(|digits| u8::from_str_radix(&digits, 16).expect("Unable to parse hex digits in data")).collect();
        let _nl = self.file.read_exact(&mut [0u8; 2]); // Read past newline
        Some(object)
    }
}