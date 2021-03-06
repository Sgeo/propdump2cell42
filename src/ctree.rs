#![allow(non_upper_case_globals)]

extern crate libloading as lib;

use std::os::raw::c_char;

use std::ffi::CString;

use std::fmt;
use std::error;


lazy_static! {
    static ref CT: lib::Library = lib::Library::new("ctreestd.dll").unwrap();
    static ref InitCTree: lib::Symbol<'static, unsafe extern "C" fn(i16, i16, i16) -> i16> = unsafe { CT.get(b"_INTREE\0").unwrap() };
    static ref AvailableFileNbr: lib::Symbol<'static, unsafe extern "C" fn(i16) -> i16> = unsafe { CT.get(b"_AVLFILNUM\0").unwrap() };
    static ref OpenCtFile: lib::Symbol<'static, unsafe extern "C" fn(i16, *const c_char, i16) -> i16> = unsafe { CT.get(b"_OPNFIL\0").unwrap() };
    static ref CloseCtFile: lib::Symbol<'static, unsafe extern "C" fn(i16, i16) -> i16> = unsafe { CT.get(b"_CLSFIL\0").unwrap() };
    static ref AddKey: lib::Symbol<'static, unsafe extern "C" fn(i16, *const u8, i32, i16) -> i16> = unsafe { CT.get(b"_ADDKEY\0").unwrap() };
    static ref NewVData: lib::Symbol<'static, unsafe extern "C" fn(i16, i32) -> i32> = unsafe { CT.get(b"_NEWVREC\0").unwrap() };
    static ref WriteVData: lib::Symbol<'static, unsafe extern "C" fn(i16, i32, *const u8, i32) -> i16> = unsafe { CT.get(b"_WRTVREC\0").unwrap() };
    static ref GetKey: lib::Symbol<'static, unsafe extern "C" fn(i16, *const u8) -> i32> = unsafe { CT.get(b"_EQLKEY\0").unwrap() };
    static ref DeleteKey: lib::Symbol<'static, unsafe extern "C" fn(i16, *const u8, i32) -> i16> = unsafe { CT.get(b"_DELCHK\0").unwrap() };
    static ref ReadVData: lib::Symbol<'static, unsafe extern "C" fn(i16, i32, *mut u8, i32) -> i16> = unsafe { CT.get(b"_RDVREC\0").unwrap() };
    static ref VDataLength: lib::Symbol<'static, unsafe extern "C" fn(i16, i32) -> i32> = unsafe { CT.get(b"_GTVLEN\0").unwrap() };
    static ref ReleaseVData: lib::Symbol<'static, unsafe extern "C" fn(i16, i32) -> i16> = unsafe { CT.get(b"_RETVREC\0").unwrap() };
    static ref GetCtFileInfo: lib::Symbol<'static, unsafe extern "C" fn(i16, i16) -> i32> = unsafe { CT.get(b"_GETFIL\0").unwrap() };
}

#[derive(Copy, Clone, Debug)]
pub enum Error {
    CTree(i16),
    OutOfSpace,
    BadKeyLength
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::CTree(num) => write!(f, "C-Tree error: {}", num),
            Error::OutOfSpace => write!(f, "Hit 2GB AW limit"),
            Error::BadKeyLength => write!(f, "Incorrectly sized key used"),
        }
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
        Err(Error::CTree(errcode))
    }
}

pub fn init() -> Result<(), Error> {
    error(unsafe {
        InitCTree(3, 2, 32)
    })
}

#[derive(Copy, Clone, Debug)]
pub struct DatAddr(i32);

#[derive(Debug)]
pub struct DatFile(i16);
#[derive(Debug)]
pub struct IdxFile(i16, usize);

impl DatFile {
    pub fn open<S: Into<Vec<u8>>>(filename: S) -> Result<Self, Error> {
        let filenum = unsafe { AvailableFileNbr(1) };
        if filenum == -1 {
            return Err(Error::CTree(-1));
        }
        let filename = CString::new(filename).unwrap();
        let result = unsafe { OpenCtFile(filenum, filename.as_ptr(), 0) };
        error(result).map(|_| DatFile(filenum))
    }
    
    fn new_v_data(&self, len: i32) -> Result<DatAddr, Error> {
        let result = unsafe {
            NewVData(self.0, len)
        };
        if result == 0 || result > (i32::max_value() - 22000) {
            if result != 0 {
                let _ = self.release_v_data(&DatAddr(result));
                Err(Error::OutOfSpace)
            } else {
                Err(Error::CTree(0))
            }
        } else {
            Ok(DatAddr(result))
        }
    }
    
    fn write_v_data(&self, addr: &DatAddr, data: &[u8]) -> Result<(), Error> {
        error(unsafe {
            WriteVData(self.0, addr.0, data.as_ptr(), data.len() as i32)
        })
    }
    
    fn read_v_data(&self, addr: &DatAddr) -> Result<Vec<u8>, Error> {
        unsafe {
            let length = VDataLength(self.0, addr.0); 
            let mut buffer = vec![0; length as usize];
            error(ReadVData(self.0, addr.0, buffer.as_mut_ptr(), buffer.len() as i32)).map(|_| buffer)
        }
    }
    
    fn release_v_data(&self, addr: &DatAddr) -> Result<(), Error> {
        unsafe {
            error(ReleaseVData(self.0, addr.0))
        }
    }
    
    
}

impl Drop for DatFile {
    fn drop(&mut self) {
        error(unsafe {
            CloseCtFile(self.0, 0)
        }).expect("Unable to correctly close a DatFile!")
    }
}

impl IdxFile {
    pub fn open<S: Into<Vec<u8>>>(filename: S) -> Result<Self, Error> {
        let filenum = unsafe { AvailableFileNbr(1) };
        if filenum == -1 {
            return Err(Error::CTree(-1));
        }
        let filename = CString::new(filename).unwrap();
        let result = unsafe { OpenCtFile(filenum, filename.as_ptr(), 0) };
        error(result).map(|_| { 
            let keylen = unsafe {
                GetCtFileInfo(filenum, 1)
            };
            IdxFile(filenum, keylen as usize)
        })
    }
    
    fn check_key(&self, key: &[u8]) -> Result<(), Error> {
        if key.len() == self.1 {
            Ok(())
        } else {
            Err(Error::BadKeyLength)
        }
    }
    
    fn add_key(&self, key: &[u8], dataddr: &DatAddr) -> Result<(), Error> {
        self.check_key(key)?;
        error(unsafe {
            AddKey(self.0, key.as_ptr(), dataddr.0, 0)
        })
    }
    
    fn get_key(&self, key: &[u8]) -> Option<DatAddr> {
        self.check_key(key).unwrap();
        let num_addr = unsafe {
            GetKey(self.0, key.as_ptr())
        };
        if num_addr == 0 {
            None
        } else {
            Some(DatAddr(num_addr))
        }
    }
    
    fn delete_key(&self, key: &[u8], addr: &DatAddr) -> Result<(), Error> {
        self.check_key(key)?;
        error(unsafe {
            DeleteKey(self.0, key.as_ptr(), addr.0)
        })
    }
}

impl Drop for IdxFile {
    fn drop(&mut self) {
        error(unsafe {
            CloseCtFile(self.0, 0)
        }).expect("Unable to correctly close an IdxFile!")
    }
}


pub fn insert_or_append(idx: &IdxFile, dat: &DatFile, key: &[u8], data: &[u8]) -> Result<(), Error> {
    let addr = dat.new_v_data(data.len() as i32)?;
    dat.write_v_data(&addr, data)?;
    let add_key_result = idx.add_key(key, &addr);
    if let Err(Error::CTree(2)) = add_key_result {
        dat.release_v_data(&addr)?;
        let old_addr = idx.get_key(key).ok_or(Error::CTree(0))?;
        let mut old_data = dat.read_v_data(&old_addr)?;
        old_data.extend_from_slice(data);
        dat.release_v_data(&old_addr)?;
        idx.delete_key(key, &old_addr)?;
        let addr = dat.new_v_data(old_data.len() as i32)?;
        dat.write_v_data(&addr, &old_data)?;
        idx.add_key(key, &addr)?;
    } else {
        add_key_result?;
    }
    Ok(())
}

