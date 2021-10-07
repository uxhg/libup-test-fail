use std::fs::{File, remove_file};
use std::future::Future;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use log::{debug, error, info, warn};
use reqwest::{StatusCode, Url};

use crate::pomdep::MvnCoord;
use crate::utils::{err, mvn_repo_util};

pub fn get_jar_name(coord: &MvnCoord) -> String {
    format!("{}-{}.jar", coord.artifact_id(), coord.version_id())
}

pub fn get_tests_jar_name(coord: &MvnCoord) -> String {
    format!("{}-{}-tests.jar", coord.artifact_id(), coord.version_id())
}

pub fn get_source_jar_name(coord: &MvnCoord) -> String {
    format!("{}-{}-sources.jar", coord.artifact_id(), coord.version_id())
}

pub fn is_file_empty(file: &PathBuf) -> bool {
    return File::open(file).unwrap().metadata().unwrap().len() == 0;
}

pub async fn get_remote_jar_to_dir(coord: &MvnCoord, dest_dir: &PathBuf) { // -> Result<usize, err::Error> {
    let file_path = dest_dir.join(mvn_repo_util::get_jar_name(&coord));
    if file_path.exists() {
        warn!("{} already exists, skip", file_path.to_str().unwrap());
        //Ok(0)
    } else {
        match File::create(&file_path) {
            Ok(f) => {
                debug!("Destination file is ready");
                //get_jar(&mvn_coord, &f).await;
                let mut writer = BufWriter::new(&f);
                get_jar_remote_wrapper(coord, &mut writer, get_jar_name).await;
            }
            Err(e) => {
                error!("Cannot create file, due to: {}", e);
                // Err(err::Error::new(err::ErrorKind::Others(e.to_string())))
            }
        }
    }
}

pub async fn get_remote_tests_jar_to_dir(coord: &MvnCoord, dest_dir: &PathBuf) {
    let file_path = dest_dir.join(mvn_repo_util::get_tests_jar_name(&coord));
    if file_path.exists() {
        warn!("{} already exists, skip", file_path.to_str().unwrap());
        // Ok(0)
    } else {
        match File::create(&file_path) {
            Ok(f) => {
                debug!("Destination file is ready");
                //get_jar(&mvn_coord, &f).await;
                let mut writer = BufWriter::new(&f);
                get_jar_remote_wrapper(coord, &mut writer, get_tests_jar_name).await;
                if is_file_empty(&file_path) {
                    remove_file(&file_path);
                    info!("{} is empty, removed", file_path.to_str().unwrap());
                }
            }
            Err(e) => {
                error!("Cannot create file, due to: {}", e);
                // Err(err::Error::new(err::ErrorKind::Others(e.to_string())))
            }
        };
    }
}

pub async fn get_remote_sources_jar_to_dir(coord: &MvnCoord, dest_dir: &PathBuf) {
    let file_path = dest_dir.join(mvn_repo_util::get_source_jar_name(&coord));
    if file_path.exists() {
        warn!("{} already exists, skip", file_path.to_str().unwrap());
        // Ok(0)
    } else {
        match File::create(&file_path) {
            Ok(f) => {
                debug!("Destination file is ready");
                //get_jar(&mvn_coord, &f).await;
                let mut writer = BufWriter::new(&f);
                get_jar_remote_wrapper(coord, &mut writer, get_source_jar_name).await;
                if is_file_empty(&file_path) {
                    remove_file(&file_path);
                    info!("{} is empty, removed", file_path.to_str().unwrap());
                }
            }
            Err(e) => {
                error!("Cannot create file, due to: {}", e);
                // Err(err::Error::new(err::ErrorKind::Others(e.to_string())))
            }
        };
    }
}

pub async fn get_jar_remote_wrapper<W: Write>(coord: &MvnCoord, out: &mut W,
                                              get_base_name: fn(&MvnCoord) -> String) {
    // e.g., https://repo1.maven.org/maven2/org/assertj/assertj-core/3.20.2/assertj-core-3.20.2-tests.jar
    let path_segments = format!("{}/{}/{}/{}",
                                &coord.group_id().replace(".", "/"),
                                &coord.artifact_id(),
                                &coord.version_id(),
                                &get_base_name(&coord));
    get_jar_by_repo_path_seg(&path_segments, out).await;
}


pub async fn get_jar_by_repo_path_seg<W: Write>(path_seg: &String, out: &mut W) {
    //let url_prefix = Url::parse("https://repo1.maven.org/maven2/").unwrap();
    let url_prefix = Url::parse("https://repo1.maven.org/maven2/").unwrap();
    let url = url_prefix.join(&path_seg).expect("Cannot join paths to compose a URL");
    // TODO: handle errors
    fetch_remote(url, out).await;
}


pub async fn fetch_remote<W: Write>(url: Url, out: &mut W) -> Result<usize, err::Error> {
    info!("Fetching file from {}", url);
    let resp = reqwest::get(url).await?;
    if resp.status() == StatusCode::NOT_FOUND.as_u16() {
        return Err(err::Error::new(err::ErrorKind::Others("HTTP Response 404".to_string())));
    }
    let contents = resp.bytes().await?;
    info!("Get file of size: {}", contents.len());
    match out.write(contents.as_ref()) {
        Ok(u) => Ok(u),
        Err(e) => Err(err::Error::new(err::ErrorKind::IOErr(e)))
    }
}
