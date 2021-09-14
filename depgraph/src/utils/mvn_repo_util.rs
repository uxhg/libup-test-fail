use std::io::{Write, BufWriter};

use log::{error, info, warn, debug};
use reqwest::Url;

use crate::pomdep::MvnCoord;
use crate::utils::{err, mvn_repo_util};
use std::path::PathBuf;
use std::fs::File;

pub fn get_jar_name(coord: &MvnCoord) -> String {
    format!("{}-{}.jar", coord.artifact_id(), coord.version_id())
}

pub fn get_tests_jar_name(coord: &MvnCoord) -> String {
    format!("{}-{}-tests.jar", coord.artifact_id(), coord.version_id())
}

pub async fn get_remote_jar_to_dir(coord: &MvnCoord, dest_dir: &PathBuf) { // -> Result<usize, err::Error> {
    let file_path = dest_dir.join(mvn_repo_util::get_jar_name(&coord));
    if file_path.exists() {
        warn!("{} already exists, skip",  file_path.to_str().unwrap());
        //Ok(0)
    } else {
        match File::create(&file_path) {
            Ok(f) => {
                debug!("Destination file is ready");
                //get_jar(&mvn_coord, &f).await;
                let mut writer = BufWriter::new(&f);
                get_jar_remote(coord, &mut writer).await;
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
        warn!("{} already exists, skip",  file_path.to_str().unwrap());
        // Ok(0)
    } else {
        match File::create(&file_path) {
            Ok(f) => {
                debug!("Destination file is ready");
                //get_jar(&mvn_coord, &f).await;
                let mut writer = BufWriter::new(&f);
                get_tests_jar_remote(coord, &mut writer).await;
            }
            Err(e) => {
                error!("Cannot create file, due to: {}", e);
                // Err(err::Error::new(err::ErrorKind::Others(e.to_string())))
            }
        };
    }
}


pub async fn get_jar_remote<W: Write>(coord: &MvnCoord, out: &mut W) {
    // e.g., https://repo1.maven.org/maven2/org/reflections/reflections/0.9.12/reflections-0.9.12.jar
    let path_segments = format!("{}/{}/{}/{}",
                                &coord.group_id().replace(".", "/"),
                                &coord.artifact_id(),
                                &coord.version_id(),
                                &get_jar_name(&coord));
    get_jar_by_repo_path_seg(&path_segments, out).await;
}

pub async fn get_tests_jar_remote<W: Write>(coord: &MvnCoord, out: &mut W) {
    // e.g., https://repo1.maven.org/maven2/org/assertj/assertj-core/3.20.2/assertj-core-3.20.2-tests.jar
    let path_segments = format!("{}/{}/{}/{}",
                                &coord.group_id().replace(".", "/"),
                                &coord.artifact_id(),
                                &coord.version_id(),
                                &get_tests_jar_name(&coord));
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
    let contents = reqwest::get(url).await?.bytes().await?;
    info!("Get file of size: {}", contents.len());
    match out.write(contents.as_ref()) {
        Ok(u) => Ok(u),
        Err(e) => Err(err::Error::new(err::ErrorKind::IOErr(e)))
    }
}
