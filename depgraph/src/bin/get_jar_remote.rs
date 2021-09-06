use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::{Path, PathBuf};

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};
use log::{debug, error, info, warn};

use depgraph::pomdep::MvnCoord;
use depgraph::utils::{mvn_repo_util, utils};
use std::env::current_dir;

#[tokio::main]
async fn main() {
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
        },
        None => current_dir().unwrap()
    };


    match matches.value_of("MvnCoord") {
        Some(c) => {
            let m = MvnCoord::from_one_string(c);
            get_jar_if_needed(&m, &alt_out_dir, storage_dir).await;
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
                Ok(ref l) => l,
                Err(_) => {
                    error!("cannot get line from {:?}", line);
                    continue;
                }
            };
            let m = MvnCoord::from_one_string(coord_str);
            get_jar_if_needed(&m, &alt_out_dir, storage_dir).await;
        }
    }
}


async fn get_jar(mvn_coord: &MvnCoord, dest_file: &File) {
    let mut writer = BufWriter::new(dest_file);
    mvn_repo_util::get_jar_remote(mvn_coord, &mut writer).await;
}

fn create_symlink(existing_path: &Path, link_target: &Path) {
    std::os::unix::fs::symlink(existing_path, link_target);
}

async fn get_jar_if_needed(mvn_coord: &MvnCoord, relative_dir: &PathBuf, storage: &Path) {
    let sub_dir_name = mvn_coord.to_string().replace(":", "--");
    // target subdir
    let target_sub_dir = relative_dir.join(&sub_dir_name);
    if !target_sub_dir.exists() {
        // check if exist in storage
        let storage_sub_dir = PathBuf::from(storage).join(sub_dir_name);
        let jar_file_path = storage_sub_dir.join(mvn_repo_util::get_jar_name(&mvn_coord));
        if !storage_sub_dir.exists() {
            // TODO: fix param
            download_to_storage(&mvn_coord, storage).await;
        }
        if !jar_file_path.exists() {
            error!("{:?} does not exist", &jar_file_path);
        }
        create_symlink(&storage_sub_dir, &target_sub_dir);
        // info!("Create dir @ {}", &path.to_str().unwrap());
        // fs::create_dir(&path);
    }
}


async fn download_to_storage(mvn_coord: &MvnCoord, storage_dir: &Path) {
    let sub_dir_name = mvn_coord.to_string().replace(":", "--");

    let sub_dir = storage_dir.join(&sub_dir_name);
    if !sub_dir.exists() {
        info!("Create dir @ {}", &sub_dir.to_str().unwrap());
        fs::create_dir(&sub_dir);
    }

    let file_path = sub_dir.join(mvn_repo_util::get_jar_name(&mvn_coord));
    if file_path.exists() {
        warn!("{} already exists",  file_path.to_str().unwrap())
    } else {
        match File::create(&file_path) {
            Ok(f) => {
                debug!("Destination file is ready");
                get_jar(&mvn_coord, &f).await;
            }
            Err(e) => {
                error!("Cannot create file, due to: {}", e);
            }
        };
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
        .get_matches()
}
