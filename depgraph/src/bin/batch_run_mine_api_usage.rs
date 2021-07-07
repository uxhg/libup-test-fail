use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};
use log::{error, info, warn};

use depgraph::api_usage::mine_api_usage;
use depgraph::utils::existing_data_utils::RepoAtVer;
use depgraph::utils::utils;
use git2::Repository;


fn main() {
    utils::init_log();
    let matches = handle_args();
    let p = matches.value_of("Input")
        .expect("Path to JSON file must be given.");
    let projects = RepoAtVer::batch_create_from_json(p);
    let stat_file_path = matches.value_of("LocalRepoStorageMap").unwrap();
    let stat_map = serde_json::from_reader::<_, HashMap<String, String>>(
        BufReader::new(File::open(stat_file_path)
            .expect(&format!("cannot open {}", stat_file_path))))
        .expect(&format!("Cannot deserialize from {}", stat_file_path));

    let max_cloned = matches.value_of("MaxClone").unwrap().parse::<i32>()
        .expect("argument to --max should be a number (i32)");
    // get workspace path
    let mut workspace = PathBuf::from(matches.value_of("WorkSpace").unwrap());

    // get output path
    let out_dir = matches.value_of("OutDir").unwrap_or("out");
    let report_file = File::create(matches.value_of("StatusReport").unwrap_or("state.report")).unwrap();
    let mut report_file_write = BufWriter::new(report_file);

    let mut counter = 0;
    for x in projects {
        if counter >= max_cloned {
            warn!("--max is set at {}, so cloning stopped early.", max_cloned);
            break
        }
        info!("Running on {:?}", x);
        let out_path = Path::new(out_dir).join(x.name());
        if !out_path.exists() {
            std::fs::create_dir_all(out_path.as_path());
        }
        if let Some(local_path) = stat_map.get(x.url()) {
            let workspace_clone_path = match Repository::clone(local_path, workspace.join(x.name())) {
                Ok(r) =>  r.path().parent().unwrap().to_owned(),
                Err(e) => {
                    error!("Clone to workspace failed, skip {}, because: {}", x.name(), e);
                    continue
                }
            };

            match mine_api_usage(workspace_clone_path.as_path(), out_path.as_path(), true,
                                 None, None, false) {
                Err(e) => {
                    error!("Error: {}", e);
                    report_file_write.write_all(format!("{} failed\n", x.name()).as_bytes())
                },
                Ok(f) => report_file_write.write_all(format!("{} status: {:?}\n", x.name(), f).as_bytes())
            };
        } else {
            warn!("Skip {}, because {} not in stat file {}", x.name(), x.url(), stat_file_path);
            continue
        }
    }
}


fn handle_args() -> ArgMatches {
    App::new("Batch Run: API Usages Mining from a list of client repos")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("Input").required(true).index(1)
            .about("A JSON file containing repo URLs"))
        .arg(Arg::new("LocalRepoStorageMap").required(true)
            .short('m').takes_value(true)
            .about("A JSON file containing location of repos"))
        .arg(Arg::new("WorkSpace").required(true)
            .short('w').takes_value(true)
            .about("Path to work space for cloning repo"))
        .arg(Arg::new("OutDir").short('o').long("out-dir").takes_value(true)
            .about("Path to the output directory"))
        .arg(Arg::new("StatusReport").short('r').long("stat-report").takes_value(true)
            .about("A file reporting success status for projects"))
        .arg(Arg::new("MaxClone").required(true).long("--max")
            .takes_value(true)
            .about("A number indicating the upper limit for this session"))
        .get_matches()
}
