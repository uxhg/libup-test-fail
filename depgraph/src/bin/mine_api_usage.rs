use std::fs;
/// This program mine API usages from client facts.
/// 1. Produce dependency facts extracted from POM
/// 2. also Origins of depgraph (multiple modules if it is a reactor)
/// 3. Call CSlicer for generating usual FuncCall/Contain facts
/// 4.
use std::io::{BufWriter, Write};
use std::path::Path;

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};

use depgraph::api_usage::mine_api_usage;
use depgraph::utils::utils;

fn main() {
    utils::init_log();
    let matches = handle_args();
    let repo_path = Path::new(matches.value_of("INPUT").unwrap());
    let out_dir = Path::new(matches.value_of("OUTPUT").unwrap_or("output"));
    if !out_dir.exists() {
        fs::create_dir(out_dir).unwrap();
    }
    mine_api_usage(repo_path, out_dir,
                   matches.is_present("Build"),
                   matches.value_of("Build-Script"),
                   matches.value_of("CSlicer"),
                   matches.is_present("JarClassMap"));
}


fn handle_args() -> ArgMatches {
    App::new("Mine library API Usages: from client facts")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("INPUT").short('i').long("input").takes_value(true)
            .about("Path to the module")
            .required(true))
        .arg(Arg::new("CSlicer").short('j').long("cslicer-jar")
            .takes_value(true).required(false)
            .about("Specify path to CSlicer JAR"))
        .arg(Arg::new("Build").short('b').long("build")
            .takes_value(false).required(false)
            .about("Build with mvn package -DskipTests at module path"))
        .arg(Arg::new("JarClassMap").long("jar-class-map")
            .takes_value(false).required(false)
            .about("Produce Jar-Class-Map facts"))
        .arg(Arg::new("Build-Script").long("build-script").takes_value(true)
            .requires("Build")
            .about("Use specified script for building"))
        .arg(Arg::new("OUTPUT").short('o').long("out-dir").takes_value(true)
            .about("Path to the output directory"))
        .get_matches()
}
