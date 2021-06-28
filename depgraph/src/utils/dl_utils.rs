/// Utilities for Datalog
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use serde::Serialize;


pub trait ToDatalogFact {
    fn to_fact(&self) -> String;
}

/// Write datalog facts
fn write_dl_to_path<'a, I, P>(rows: I, path: P) where I: Iterator<Item=&'a dyn ToDatalogFact>, P: AsRef<Path>
{
    let mut w = BufWriter::new(File::create(path).unwrap());
    for r in rows {
        w.write_all(r.to_fact().as_bytes());
    }
}