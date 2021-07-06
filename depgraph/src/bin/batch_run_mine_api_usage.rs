use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};
use log::{info};

use depgraph::utils::utils;
use depgraph::utils::existing_data_utils::RepoAtVer;

fn main() {
    utils::init_log();
    let matches = handle_args();
    let p = matches.value_of("Input")
        .expect("Path to JSON file must be given.");
    let projects = RepoAtVer::batch_create_from_json(p);
    for x in projects {
        info!("Running on {:?}", x);
        // mine_api_usage()
    }
}


fn handle_args() -> ArgMatches {
    App::new("Batch Run: API Usages Mining from a list of client repos")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("Input").required(true).index(1)
            .about("A JSON file containing repo URLs"))
        .arg(Arg::new("RepoStorage").required(true).takes_value(true)
            .about("A JSON file containing location of repos"))
        .get_matches()
}
