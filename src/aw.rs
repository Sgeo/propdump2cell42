extern crate byteorder;

use std::io::{self, Write};
use byteorder::{ByteOrder, LE, WriteBytesExt};

#[derive(Debug, Clone, Default)]
pub struct Object {
    pub type_: i32,
    pub id: i32,
    pub number: i32,
    pub citnum: i32,
    pub time: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub yaw: i16,
    pub tilt: i16,
    pub roll: i16,
    pub name: Vec<u8>,
    pub desc: Vec<u8>,
    pub action: Vec<u8>,
    pub data: Vec<u8>
}

#[derive(Debug, Copy, Clone)]
pub struct Location {
    pub cell_x: i16,
    pub cell_z: i16,
    pub obj_x: i16,
    pub obj_y: i32,
    pub obj_z: i16
}

impl Object {
    pub fn location(&self) -> Location {
        Location {
            cell_x: (self.x/1000) as i16,
            cell_z: (self.z/1000) as i16,
            obj_x: (self.x%1000) as i16,
            obj_z: (self.z%1000) as i16,
            obj_y: self.y
        }
    }
    pub fn write<W: Write>(&self, mut w: W) -> io::Result<()> {
        let loc = self.location();
        w.write_i32::<LE>(self.type_)?;
        w.write_i32::<LE>(self.id)?;
        w.write_i32::<LE>(self.number)?;
        w.write_i32::<LE>(self.citnum)?;
        w.write_i32::<LE>(self.time)?;
        w.write_i16::<LE>(loc.obj_x)?;
        w.write_i32::<LE>(loc.obj_y)?;
        w.write_i16::<LE>(loc.obj_z)?;
        w.write_i16::<LE>(self.yaw)?;
        w.write_i16::<LE>(self.tilt)?;
        w.write_i16::<LE>(self.roll)?;
        w.write_u8(self.name.len() as u8)?;
        w.write_u8(self.desc.len() as u8)?;
        w.write_u8(self.action.len() as u8)?;
        w.write_u16::<LE>(self.data.len() as u16)?;
        w.write_all(&self.name)?;
        w.write_all(&self.desc)?;
        w.write_all(&self.action)?;
        w.write_all(&self.data)?;
        Ok(())
    } 
}