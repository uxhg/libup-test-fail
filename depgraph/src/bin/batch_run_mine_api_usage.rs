use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};
use serde::{Deserialize, Serialize};

use depgraph::utils::utils;

fn main() {
    let matches = handle_args();
    let p = matches.value_of("INPUT")
        .expect("Path to JSON file must be given.");
    let projects = read_all_repos(p);
    for x in projects {
        println!("{:?}", x);
    }
}


#[derive(Deserialize, Debug)]
struct RepoAtVer {
    name: String,
    sha: String,
    url: String,
    tag: String,
}


fn read_all_repos<P: AsRef<Path>>(file_path: P) -> Vec<RepoAtVer> {
    let reader = utils::load_json(file_path.as_ref());
    serde_json::from_reader::<_, Vec<RepoAtVer>>(reader)
        .expect("Failed to deserialize from JSON to a list of repos.")
}

fn handle_args() -> ArgMatches {
    App::new("Batch Run: API Usages Mining from a list of client repos")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("INPUT").required(true).index(1)
            .about("Path to the JSON file containing repo URLs"))
        .get_matches()
}
