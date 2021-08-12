use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};
use log::{error, info, warn, debug};

use depgraph::pomdep::MvnCoord;
use depgraph::utils::{mvn_repo_util, utils};

#[tokio::main]
async fn main() {
    utils::init_log();
    let matches = handle_args();
    let mvn_coord = MvnCoord::from_one_string(matches.value_of("MvnCoord").unwrap());
    let alt_out_dir = matches.value_of("AltDir");

    match create_download_dest(&mvn_coord, alt_out_dir) {
        Ok(f) => {
            debug!("Destination file is ready");
            get_jar(&mvn_coord, &f).await;
        },
        Err(e) => {error!("Cannot create file");}
    };
}

async fn get_jar(mvn_coord: &MvnCoord, dest_file: &File) {
    let mut writer = BufWriter::new(dest_file);
    info!("test");
    mvn_repo_util::get_jar_remote(mvn_coord, &mut writer).await;
}

fn create_download_dest(mvn_coord: &MvnCoord, relative_dir: Option<&str>) -> std::io::Result<File> {
    let sub_dir_name = mvn_coord.group_artifact_id().replace(":", "--");
    let path = match relative_dir {
        Some(p) => PathBuf::from(p).join(&sub_dir_name),
        None => PathBuf::from(&sub_dir_name)
    };
    if !path.exists() {
        info!("Create dir @ {}", &path.to_str().unwrap());
        fs::create_dir(&path);
    }
    let file_name = mvn_repo_util::get_jar_name(&mvn_coord);
    File::create(path.join(file_name))
}


fn handle_args() -> ArgMatches {
    App::new("Download jars from maven repositories.")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("MvnCoord").required(true).index(1)
            .about("Maven coordinate, e.g., org.jsoup:jsoup:1.14.1"))
        .arg(Arg::new("AltDir").long("--dir").takes_value(true)
            .about("Specify output dir instead of cwd"))
        .get_matches()
}
