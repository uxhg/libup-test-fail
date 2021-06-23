/// This program mine API usages from client facts.

use depgraph::utils::utils;
use clap::{ArgMatches, App, Arg};
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;
use depgraph::jar_class_map::MvnModule;

fn main() {
    utils::init_log();
    let matches = handle_args();
    let mod_path = Path::new(matches.value_of("INPUT").unwrap());
    let mod_name = mod_path.file_name().unwrap().to_str().unwrap();

    let cslicer_cfg_path = mod_path.join(format!("{}.properties", mod_name));
    match utils::create_cslicer_config(mod_path, &mut BufWriter::new(File::create(cslicer_cfg_path).unwrap())) {
        Err(e) => println!("{}", e),
        Ok(_) => ()
    }
    let mvn_mod = MvnModule::new(mod_name, mod_path.to_str().unwrap());
}

fn handle_args() -> ArgMatches {
    App::new("Mine library API Usages: from client facts")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("INPUT").short('i').long("input").takes_value(true)
            .about("Path to the module")
            .required(true)).get_matches()
}
