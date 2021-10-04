use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};
use log::{error, info, warn};

use depgraph::utils::existing_data_utils::RepoAtVer;
use depgraph::utils::utils;

pub mod rank_api;

fn main() {
    utils::init_log();
    let matches = handle_args();
    let project_list_file = matches.value_of("Input").unwrap();
    let repo_storage_loc = matches.value_of("Storage").unwrap();
    let stat_file = Path::new(matches.value_of("LocalRepoStorageMap").unwrap());
    let max_cloned = match matches.is_present("MaxClone") {
        true => matches.value_of("MaxClone").unwrap().parse::<u32>()
            .expect("argument to --max should be a number (u32)"),
        false => 0,
    };
    // read cloned repos
    let mut existing_stat = read_existing_stat(stat_file.as_ref());

    let projects = RepoAtVer::batch_create_from_json(project_list_file);
    let mut counter = 0;
    for x in projects {
        if max_cloned != 0 && counter >= max_cloned {
            warn!("--max is set at {}, so cloning stopped early.", max_cloned);
            break
        }
        let repo_url = x.url();
        if existing_stat.contains_key(repo_url) {
            info!("Skip {}, already exists @ {}", repo_url, existing_stat.get(repo_url)
                .unwrap_or(&String::from("NOT_FOUND")));
            continue
        }
        info!("Cloning {} from {}", x.name(), repo_url);
        if let Some(r) = utils::clone_remote(x.url(), Path::new(repo_storage_loc)) {
            existing_stat.insert(repo_url.to_string(), String::from(r.path().parent().unwrap().to_str().unwrap_or_default()));
            counter += 1;
        }
    }
    // write back existing_stat to file
    let stat_writer = BufWriter::new(File::create(stat_file)
        .expect("Cannot open or create stat file"));
    serde_json::ser::to_writer_pretty(stat_writer, &existing_stat);
}

fn read_existing_stat(stat_file: &Path) -> HashMap<String, String> {
    match stat_file.exists() {
        true => {
            let stat_reader = utils::load_json(stat_file);
            serde_json::from_reader::<_, HashMap<String, String>>(stat_reader)
                .expect("Local storage map file should be a hashmap between repo_url and local location")
        },
        false => {
            info!("{} does not exist, will create new", stat_file.to_str().unwrap_or_default());
            HashMap::new()
        }
    }
}



fn handle_args() -> ArgMatches {
    App::new("Make local repo store: clone --bare to local for future use")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("Input").required(true).index(1)
            .about("A JSON file containing repo URLs"))
        .arg(Arg::new("Storage").required(true).short('s')
            .takes_value(true)
            .about("The path to local repo storage"))
        .arg(Arg::new("LocalRepoStorageMap").required(true).short('m')
            .takes_value(true)
            .about("A JSON file mapping repo URL to local storage path"))
        .arg(Arg::new("MaxClone").long("--max")
            .takes_value(true)
            .about("A number indicating the upper limit of cloning in this session"))
        .get_matches()
}
