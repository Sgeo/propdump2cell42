extern crate libloading as lib;

use std::os::raw::{c_char, c_void};

use std::ffi::CString;

use std::fmt;
use std::error;

lazy_static! {
    static ref CT: lib::Library = lib::Library::new("ctreestd.dll").unwrap();
    static ref InitCTree: lib::Symbol<'static, unsafe extern "C" fn(i16, i16, i16) -> i16> = unsafe { CT.get(b"_INTREE\0").unwrap() };
    static ref OpenCtFile: lib::Symbol<'static, unsafe extern "C" fn(i16, *mut c_char, i16) -> i16> = unsafe { CT.get(b"_OPNFIL\0").unwrap() };
    static ref CloseCtFile: lib::Symbol<'static, unsafe extern "C" fn(i16, i16) -> i16> = unsafe { CT.get(b"_CLSFIL\0").unwrap() };
    static ref GetCtFileInfo: lib::Symbol<'static, unsafe extern "C" fn(i16, i16) -> i32> = unsafe { CT.get(b"_GETFIL\0").unwrap() };
    static ref FirstKey: lib::Symbol<'static, unsafe extern "C" fn(i16, *mut u8) -> i32> = unsafe { CT.get(b"_FRSKEY\0").unwrap() };
    static ref NextKey: lib::Symbol<'static, unsafe extern "C" fn(i16, *mut u8) -> i32> = unsafe { CT.get(b"_NXTKEY\0").unwrap() };
    static ref VDataLength: lib::Symbol<'static, unsafe extern "C" fn(i16, i32) -> i32> = unsafe { CT.get(b"_GTVLEN\0").unwrap() };
    static ref ReadVData: lib::Symbol<'static, unsafe extern "C" fn(i16, i32, *mut u8, i32) -> i16> = unsafe { CT.get(b"_RDVREC\0").unwrap() };
}

#[derive(Copy, Clone, Debug)]
pub struct Error(i16);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "C-Tree error: {}", self.0)
    }
}

impl error::Error for Error {
    fn description(&self) -> &'static str {
        "C-Tree error"
    }
}

fn error(errcode: i16) -> Result<(), Error> {
    if errcode == 0 {
        Ok(())
    } else {
        Err(Error(errcode))
    }
}

pub fn init() -> Result<(), Error> {
    error(unsafe {
        InitCTree(3, 2, 32)
    })
}

pub struct DatFile(i16)


