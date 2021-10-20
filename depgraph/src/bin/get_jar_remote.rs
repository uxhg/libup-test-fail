use std::env::current_dir;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};
use log::{error, info};

use depgraph::pomdep::MvnCoord;
use depgraph::utils::mvn_repo_util;
use depgraph::utils::mvn_repo_util::JarNameFn;
use depgraph::utils::utils;
use std::collections::HashMap;

fn main() {
    utils::init_log();
    let matches = handle_args();
    let storage_dir = Path::new(matches.value_of("JarStore").unwrap());
    let alt_out_dir = match matches.value_of("AltDir") {
        Some(p) => {
            let path = PathBuf::from(p);
            if !path.exists() {
                fs::create_dir(&path);
            }
            path
        }
        None => current_dir().unwrap()
    };

    let mut rt = tokio::runtime::Runtime::new().unwrap();


    let get_jar_fn: HashMap<&str, JarNameFn> = [
        ("main", mvn_repo_util::jar_name as JarNameFn),
        ("test", mvn_repo_util::tests_jar_name as JarNameFn),
        ("source", mvn_repo_util::source_jar_name as JarNameFn),
        ("test-source", mvn_repo_util::test_source_jar_name as JarNameFn),
    ].iter().cloned().collect();

    // get_jar_fn.get(x)

    let selections = match matches.values_of("Selection") {
        Some(o) => {
            o.map(|x| get_jar_fn.get(x).unwrap_or_else(
                || panic!("{} is not in the list of selections", x)))
                .cloned().collect::<Vec<JarNameFn>>()
        },
        None => vec![mvn_repo_util::jar_name as JarNameFn,
                     mvn_repo_util::tests_jar_name as JarNameFn]
    };

    match matches.value_of("MvnCoord") {
        Some(c) => {
            let m = MvnCoord::from_one_string(c)
                .expect("Cannot convert the given str to a maven coordinate");
            let async_getter = async { get_jar_if_needed(&m, &alt_out_dir, storage_dir, &selections).await; };
            rt.block_on(async_getter);
        }
        None => ()
    };
    //let mvn_coord = MvnCoord::from_one_string(matches.value_of("MvnCoord").unwrap());

    // read a file containing a list of mvn coordinates
    if let Some(s) = matches.value_of("CoordList") {
        let f = std::fs::File::open(Path::new(s))
            .expect(&format!("Cannot open file {}", s));
        for line in BufReader::new(f).lines() {
            let coord_str = match line {
                Ok(ref l) => {
                    if l.starts_with("#") || l.trim().is_empty() {
                        // ignore comments and blank lines
                        continue;
                    } else if l.contains(char::is_whitespace) {
                        let name = l.split_whitespace().collect::<Vec<&str>>()[1];
                        info!("Read name {} from gradle (short) declaration", name);
                        &name[1..name.len() - 1]
                    } else {
                        info!("Read name {}", l);
                        l
                    }
                }
                Err(_) => {
                    error!("cannot get line from {:?}", line);
                    continue;
                }
            };
            match MvnCoord::from_one_string(coord_str) {
                Some(m) => {
                    let async_getter = async { get_jar_if_needed(&m, &alt_out_dir, storage_dir, &selections).await; };
                    rt.block_on(async_getter);
                }
                None => {
                    error!("Cannot convert {} to a maven coordinate", coord_str);
                    continue;
                }
            }
        }
    }
}


fn create_symlink(existing_path: &Path, link_target: &Path) {
    info!("Create symlink from {:?} to {:?}", existing_path, link_target);
    if std::os::unix::fs::symlink(existing_path, link_target).is_err() {
        // TODO: handle error
        error!("Cannot create symlink {} -> {}",
               existing_path.to_str().unwrap(),
               link_target.to_str().unwrap())
    }
}

async fn get_jar_if_needed(mvn_coord: &MvnCoord, relative_dir: &PathBuf, storage: &Path,
                           selections: &Vec<fn(&MvnCoord) -> String>) {
    let sub_dir_name = mvn_coord.to_string().replace(":", "--");
    let storage_sub_dir = storage.join(&sub_dir_name);
    if !storage_sub_dir.exists() {
        info!("Create dir @ {}", &storage_sub_dir.to_str().unwrap());
        if fs::create_dir(&storage_sub_dir).is_err() {
            // TODO: handle error
            error!("Cannot create dir @ {}", &storage_sub_dir.to_str().unwrap())
        }
    }

    for get_jar_name_fn in selections {
        mvn_repo_util::get_remote_jars_to_dir(mvn_coord, &storage_sub_dir, get_jar_name_fn).await;
    }

    //let jar_file_path = storage_sub_dir.join(mvn_repo_util::get_jar_name(&mvn_coord));
    //if !jar_file_path.exists() {
    //    error!("{:?} does not exist", &jar_file_path);
    //}
    let target_sub_dir = relative_dir.join(&sub_dir_name);
    if !target_sub_dir.exists() {
        create_symlink(&storage_sub_dir, &target_sub_dir);
    }
}


fn handle_args() -> ArgMatches {
    App::new("Download jars from maven repositories.")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("MvnCoord").short('c').takes_value(true)
            .about("Maven coordinate, e.g., org.jsoup:jsoup:1.14.1"))
        .arg(Arg::new("CoordList").short('l').long("list").takes_value(true)
            .about("A list of maven coordinates, each on one row"))
        .arg(Arg::new("AltDir").long("dir").takes_value(true)
            .about("Specify output dir instead of cwd"))
        .arg(Arg::new("JarStore").long("storage").takes_value(true).required(true)
            .about("Specify existing jar storage"))
        .arg(Arg::new("Selection").long("sel").takes_value(true).multiple(true)
            .about("Select what jars to download"))
        .get_matches()
}
