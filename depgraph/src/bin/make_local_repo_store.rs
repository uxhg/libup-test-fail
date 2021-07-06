use std::path::Path;
use std::path::PathBuf;

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};
use git2::Repository;

use log::{error,info,warn};
use depgraph::utils::existing_data_utils::RepoAtVer;
use depgraph::utils::utils;
use url::Url;
use std::collections::HashMap;
use std::io::BufWriter;
use std::fs::File;

fn main() {
    let matches = handle_args();
    let project_list_file = matches.value_of("Input").unwrap();
    let repo_storage_loc = matches.value_of("Storage").unwrap();
    let stat_file = Path::new(matches.value_of("Stat").unwrap());

    // read cloned repos
    let mut existing_stat = read_existing_stat(stat_file.as_ref());

    let projects = RepoAtVer::batch_create_from_json(project_list_file);
    for x in projects {
        let repo_url = x.url();
        if existing_stat.contains_key(repo_url) {
            info!("Skip {}, already exists @ {}", repo_url, existing_stat.get(repo_url)
                .unwrap_or(&String::from("NOT_FOUND")));
            continue
        }
        info!("Cloning on {:?}", x);
        if let Some(r) = clone_remote(x.url(), Path::new(repo_storage_loc), stat_file) {
            existing_stat.insert(repo_url.to_string(), String::from(r.path().parent().unwrap().to_str().unwrap_or_default()));
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
                .expect("Stat file should be a hashmap between repo_url and local location")
        },
        false => {
            info!("{} does not exist, will create new", stat_file.to_str().unwrap_or_default());
            HashMap::new()
        }
    }
}


fn clone_remote(url: &str, local_path: &Path, stat_file: &Path) -> Option<Repository> {
    let mut clone_to_path = PathBuf::from(local_path);
    match Url::parse(url) {
        Err(e) => {
            error!("Cannot parse {}, errors: {}", url, e);
            clone_to_path.join("un-organized");
        },
        Ok(u) => {
            if let Some(s) = u.path_segments() {
                clone_to_path.extend(s);
            } else {
                warn!("Cannot split path for {}", url);
                clone_to_path.join("un-organized");
            }
        }
    };

    match Repository::clone(url, clone_to_path) {
        Ok(repo) => {
            Some(repo)
        },
        Err(e) => {
            error!("Failed to clone: {}", e);
            None
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
        .arg(Arg::new("Stat").required(true).short('t')
            .takes_value(true)
            .about("A JSON file mapping repo URL to local storage path"))
        .get_matches()
}
