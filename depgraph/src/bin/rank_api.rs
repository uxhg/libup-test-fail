use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};
use log::{error, info, warn};

use depgraph::utils::err;
use depgraph::utils::utils;

fn main() {
    utils::init_log();
    let matches = handle_args();
    let input_path = Path::new(matches.value_of("Input").unwrap());
    let mut results: HashMap<String, HashSet<String>> = HashMap::new();
    let files = match utils::list_dir_non_recur(input_path) {
        Ok(x) => x,
        Err(e) => panic!("Cannot list files, error: {}", e)
    };
    for f_path in files.iter() {
        let f_path_str = &f_path.to_str().unwrap_or_default();
        info!("Reading file @ {}", f_path_str);
        let f = match std::fs::File::open(f_path) {
            Ok(o) => o,
            Err(e) => {
                error!("Cannot open {}: {}", f_path_str, e.to_string());
                continue;
            }
        };
        let file_name = match f_path.file_name() {
            Some(f) => String::from(f.to_str().unwrap()),
            None => {
                warn!("Cannot get file name of {}", &f_path.to_str().unwrap_or_default());
                continue;
            }
        };
        for line in BufReader::new(f).lines() {
            let api_name = match line {
                Ok(l) => l,
                Err(ref e) => {
                    error!("Cannot parse line {:?}, error: {}", &line, e);
                    continue;
                }
            };
            if results.contains_key(&api_name) {
                if let Some(s) = results.get_mut(&api_name) {
                    s.insert(file_name.clone());
                }
            } else {
                results.insert(api_name, vec![file_name.clone()].into_iter().collect::<HashSet<String>>());
            }
        }
    }

    let ranks_writer = BufWriter::new(File::create("ranks.json")
        .expect("Cannot open or create stat file"));
    let mut sorted_results: Vec<(String, HashSet<String>)> = results.into_iter().collect();
    sorted_results.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    serde_json::ser::to_writer_pretty(ranks_writer, &sorted_results);
}


fn handle_args() -> ArgMatches {
    App::new("Find popular APIs.")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("Input").required(true).index(1)
            .about("A directory, where each file contains a list of APIs."))
        .get_matches()
}
