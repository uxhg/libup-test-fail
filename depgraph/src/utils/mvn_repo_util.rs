use std::collections::HashMap;
use std::fs::{File, remove_file};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use log::{debug, error, info, warn};
use reqwest::{StatusCode, Url};
use tokio::task::block_in_place;

use crate::pomdep::MvnCoord;
use crate::utils::{err, mvn_repo_util};
use crate::utils::err::Error;

pub fn jar_name(coord: &MvnCoord) -> String {
    format!("{}-{}.jar", coord.artifact_id(), coord.version_id())
}

pub fn tests_jar_name(coord: &MvnCoord) -> String {
    format!("{}-{}-tests.jar", coord.artifact_id(), coord.version_id())
}

pub fn source_jar_name(coord: &MvnCoord) -> String {
    format!("{}-{}-sources.jar", coord.artifact_id(), coord.version_id())
}

pub fn test_source_jar_name(coord: &MvnCoord) -> String {
    format!("{}-{}-test-sources.jar", coord.artifact_id(), coord.version_id())
}

pub fn is_file_empty(file: &PathBuf) -> bool {
    return File::open(file).unwrap().metadata().unwrap().len() == 0;
}

pub type JarNameFn = fn(&MvnCoord) -> String;

pub async fn get_remote_jars_to_dir(coord: &MvnCoord, dest_dir: &PathBuf,
                                    get_name_fn: &fn(&MvnCoord) -> String) {
    let file_path = dest_dir.join(get_name_fn(&coord));
    if file_path.exists() {
        warn!("{} already exists, skip", file_path.to_str().unwrap());
        //Ok(0)
    } else {
        match File::create(&file_path) {
            Ok(f) => {
                debug!("Destination file is ready");
                //get_jar(&mvn_coord, &f).await;
                let mut writer = BufWriter::new(&f);
                get_jar_remote_wrapper(coord, &mut writer, get_name_fn).await;
                // if is_file_empty(&file_path) {
                //     remove_file(&file_path).unwrap_or_else(|_| panic!("Cannot remove file: {:?}", &file_path));
                //     info!("{} is empty, removed", file_path.to_str().unwrap());
                // }
            }
            Err(e) => {
                error!("Cannot create file, due to: {}", e);
                // Err(err::Error::new(err::ErrorKind::Others(e.to_string())))
            }
        }
    }
}


pub async fn get_jar_remote_wrapper<W: Write>(coord: &MvnCoord, out: &mut W,
                                              get_base_name: &fn(&MvnCoord) -> String) {
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
    match fetch_remote(url, out).await {
        Ok(_) => {}
        Err(e) => { error!("Error when fetching file: {}\n{}", path_seg, e) }
    }
}


pub async fn fetch_remote<W: Write>(url: Url, out: &mut W) -> Result<usize, err::Error> {
    info!("Fetching file from {}", url);
    let resp = reqwest::get(url).await?;
    if resp.status() == StatusCode::NOT_FOUND.as_u16() {
        return Err(err::Error::new(err::ErrorKind::Others("HTTP Response 404".to_string())));
    }
    let contents = resp.bytes().await?;
    info!("Get file size: {}", contents.len());
    match out.write(contents.as_ref()) {
        Ok(u) => Ok(u),
        Err(e) => Err(err::Error::new(err::ErrorKind::IOErr(e)))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::utils::mvn_repo_util::is_file_empty;

    #[test]
    fn test_is_file_empty() {
        assert_eq!(is_file_empty(&PathBuf::from("/home/wuxh/Downloads/cas-server-support-ehcache-monitor-5.3.0-RC4.jar")), false);
    }
}