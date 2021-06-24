use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::process::{Command, Stdio};

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};
use log::{error, info, warn};

use depgraph::jar_class_map::MvnModule;
/// This program mine API usages from client facts.

use depgraph::utils::utils;

fn main() {
    utils::init_log();
    let matches = handle_args();
    let mod_path = Path::new(matches.value_of("INPUT").unwrap());
    let mod_name = mod_path.file_name().unwrap().to_str().unwrap();
    let cslicer_jar_path = matches.value_of("CSlicer").unwrap();

    let cslicer_cfg_path = mod_path.join(format!("{}.properties", mod_name));
    // let cslicer_cfg_path = Path::new("cslicer.properties");
    match utils::create_cslicer_config(mod_path, &mut BufWriter::new(File::create(&cslicer_cfg_path).unwrap())) {
        Err(e) => panic!("Abort because of creation of CSlicer configuration failed. Because: {}", e),
        Ok(_) => info!("CSlicer configuration file created successfully @ {}", cslicer_cfg_path.to_str().unwrap_or_default())
    }
    // let mvn_mod = MvnModule::new(mod_name, mod_path.to_str().unwrap());
    // if !mvn_mod.mvn_pkg_skiptests() {
    //     panic!("Abort because of mvn package failure.")
    // }
    match Command::new("java").arg("-jar").arg(cslicer_jar_path)
        .arg("-e").arg("dl").arg("-ext").arg("dep").arg("-c").arg(&cslicer_cfg_path)
        .current_dir(mod_path).stderr(Stdio::piped()).output() {
        Ok(cslicer_cmd) => {
            if cslicer_cmd.stderr.len() != 0 {
                warn!("Errors in running CSlicer: {}", std::str::from_utf8(&cslicer_cmd.stderr).unwrap());
            }
            info!("Facts generated inside {}", mod_path.join(".facts").to_str().unwrap())
        }
        Err(e) => error!("Errors when trying to run [mvn package]: {}", e)
    }
}

fn handle_args() -> ArgMatches {
    App::new("Mine library API Usages: from client facts")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("INPUT").short('i').long("input").takes_value(true)
            .about("Path to the module")
            .required(true))
        .arg(Arg::new("CSlicer").short('j').long("cslicer-jar")
            .takes_value(true)
            .about("Specify path to CSlicer JAR"))
        .get_matches()
}
