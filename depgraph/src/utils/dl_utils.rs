/// Utilities for Datalog
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::pomdep::MvnCoord;

pub trait ToDatalogFact {
    fn to_fact(&self) -> String;
}

impl ToDatalogFact for MvnCoord {
    fn to_fact(&self) -> String {
        self.to_dl_string()
    }
}


/// Write datalog facts
pub fn write_dl_to_path<'a, I, P>(rows: I, path: P) -> Result<(), std::io::Error>
    where I: Iterator<Item=&'a dyn ToDatalogFact>, P: AsRef<Path> {
    let mut w = BufWriter::new(File::create(path)?);
    for r in rows.into_iter() {
        w.write_all(r.to_fact().as_bytes()).unwrap();
    }
    Ok(())
}