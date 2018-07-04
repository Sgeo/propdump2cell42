use failure;
use aw::Object;

pub struct Teleports {
    regions: Vec<((i16, i16), (i16, i16))>
}

fn coord_to_num<S: AsRef<str>>(coord: S) -> Result<i16, failure::Error> {
    use std::str::FromStr;

    let coord = coord.as_ref();
    let (digits, indicator) = coord.split_at(coord.len() - 1);
    let indicator = indicator.to_uppercase();
    let floating = f32::from_str(digits)?;
    let integer = floating as i16;
    if indicator == "N" || indicator == "W" {
        return Ok(integer);
    } else if indicator == "S" || indicator == "E" {
        return Ok(-integer);
    } else {
        bail!("Unable to process coordinate in teleport file!");
    }
}

fn bounds(coord: i16, radius: i16) -> (i16, i16) {
    (coord.saturating_sub(radius), coord.saturating_add(radius))
}

impl Teleports {
    pub fn from_file<P: AsRef<::std::path::Path>>(path: P, radius: i16) -> Result<Self, failure::Error> {
        use std::fs::File;
        use std::io::prelude::*;
        use std::io::BufReader;
        
        let mut this = Teleports {
            regions: Vec::new()
        };
        
        let file = File::open(path)?;
        let buffer = BufReader::new(file);
        
        for line in buffer.lines() {
            let mut line = line?;
            let mut coords = line.split(':').next().expect("Unable to split on : in teleport fille!");
            let mut data = coords.split(' ');
            let _world = data.next();
            let ns = data.next();
            let ew = data.next();
            ensure!(ns.is_some() && ew.is_some(), "Unable to process line in teleport file!");
            let z = coord_to_num(ns.unwrap())?;
            let x = coord_to_num(ew.unwrap())?;
            this.regions.push((bounds(x, radius), bounds(z, radius)));
        }
        
        Ok(this)
    }

    pub fn contains(&self, object: &Object) -> bool {
        let location = object.location();
        for ((min_x, max_x), (min_z, max_z)) in &self.regions {
            if min_x <= &location.cell_x && &location.cell_x <= max_x && min_z <= &location.cell_z && &location.cell_z <= max_z {
                return true;
            }
        }
        return false;
    }
}

