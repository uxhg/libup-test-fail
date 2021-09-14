use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};
use git2::Repository;
use log::{error, info, warn};

use depgraph::api_usage::mine_api_usage;
use depgraph::utils::existing_data_utils::RepoAtVer;
use depgraph::utils::err;
use depgraph::utils::utils;

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

    let max_cloned = match matches.is_present("MaxClone") {
        true => matches.value_of("MaxClone").unwrap().parse::<u32>()
            .expect("argument to --max should be a number (u32)"),
        false => 0,
    };
    // get workspace path
    let mut workspace = PathBuf::from(matches.value_of("WorkSpace").unwrap());

    // get output path
    let out_dir = matches.value_of("OutDir").unwrap_or("out");
    let report_file = File::create(matches.value_of("StatusReport").unwrap_or("state.report")).unwrap();
    let mut report_file_write = BufWriter::new(report_file);

    let mut counter = 0;
    let mut lib_usage: HashMap<String, HashSet<String>> = HashMap::new();
    for x in projects {
        if max_cloned != 0 && counter >= max_cloned {
            warn!("--max is set at {}, so cloning stopped early.", max_cloned);
            break;
        }
        counter += 1;
        info!("Running on {:?}", x);
        let out_path = Path::new(out_dir).join(x.name());
        if !out_path.exists() {
            std::fs::create_dir_all(out_path.as_path());
        }
        if stat_map.get(x.url()).is_none() {
            warn!("Skip {}, because {} not in stat file {}", x.name(), x.url(), stat_file_path);
            continue;
        }
        let local_path = stat_map.get(x.url()).unwrap();
        let workspace_clone_path = workspace.join(x.name());
        //add let = , get Repo object to be used for checking out
        let repo = match workspace_clone_path.exists() {
            false => {// not exist, then clone
                match Repository::clone(local_path, workspace.join(x.name())) {
                    Err(e) => {
                        error!("Clone to workspace failed, skip {}, because: {}", x.name(), e);
                        continue;
                    }
                    Ok(r) => r
                }
            }
            true => {
                match Repository::open(&workspace_clone_path) {// exist, check remote url
                    Ok(r) => {
                        match r.remotes() {
                            Ok(ra) => {// check remote url
                                for x in ra.iter().filter(|x| x.is_some())
                                    .map(|x| x.unwrap().to_string()).collect::<HashSet<String>>() {
                                    let remote_url = String::from(r.find_remote(&x).unwrap().url().unwrap_or_default());
                                    info!("remote {} -> {}", &x, &remote_url);
                                }
                                r
                            }
                            Err(e) => {
                                warn!("Cannot read remotes of repo @ {}", workspace_clone_path.to_str().unwrap_or_default());
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        error!("{} exists and does not seems to be a repo", workspace_clone_path.to_str().unwrap_or_default());
                        continue;
                    }
                }
            }
        };

        // checkout to latest tag
        let latest_tag = String::from_utf8(Command::new("git").arg("describe").arg("--abbrev=0")
            .current_dir(&workspace_clone_path).output().unwrap().stdout)
            .unwrap_or_default().trim_end().to_string();
        match latest_tag.is_empty() {
            true => {
                info!("Cannot find latest tag, build on current HEAD");
            }
            false => {
                info!("Latest tag: {}", &latest_tag);
                match repo.revparse_ext(&latest_tag) {
                    Ok((c, reference)) => {
                        info!("{:?}", &c);
                        match repo.checkout_tree(&c, None) {
                            Ok(_) => {
                                info!("Checkout to tag: {}", &latest_tag);
                                match reference {
                                    // gref is an actual reference like branches or tags
                                    Some(gref) => repo.set_head(gref.name().unwrap()),
                                    // this is a commit, not a reference
                                    None => repo.set_head_detached(c.id()),
                                }.expect("Failed to set HEAD");
                            }
                            Err(checkout_err) => {
                                error!("Cannot checkout to {:?}", c)
                            }
                        }
                    }
                    Err(e) => error!("Cannot parse {}, thus not checkout to it", &latest_tag)
                }
            }
        }


        if matches.is_present("CodeQLCreateDB") {
            let db_name = format!("{}.db", x.name());
            match Command::new("codeql").arg("database")
                .arg("create").arg(db_name).arg("--language=java").current_dir(&workspace_clone_path)
                .stdout(Stdio::piped()).stderr(Stdio::piped())
                .output() {
                    Ok(c) => {
                        if !c.status.success() {
                            error!("{}: {}", err::ErrorKind::ExtCommandFailure(String::from("codeql database create")),
                            std::str::from_utf8(&c.stderr).unwrap());
                        }
                    },
                    Err(e) => {
                        error!("{}: {}", err::ErrorKind::CallToExtCommandErr(String::from("codeql database create")), e);
                    }
                }
        }

        if !matches.is_present("Heavy") {
            // skip all following heavy operations
            continue
        }
        match mine_api_usage(workspace_clone_path.as_path(), out_path.as_path(), false,
                             None, None, false, &mut lib_usage) {
            Err(e) => {
                error!("Error: {}", e);
                if let Err(e) = report_file_write.write_all(format!("{} failed\n", x.name()).as_bytes()) {
                    error!("Write report file failed: {}", e);
                }
            }
            Ok(f) => {
                if let Err(e) = report_file_write.write_all(format!("{} status: {:?}\n", x.name(), f).as_bytes()) {
                    error!("Write report file failed: {}", e);
                }
            }
        };
        // create new file and re-write each time, can be optimized
        let ranks_writer = BufWriter::new(File::create("rank_lib.json")
            .expect("Cannot open or create rank_lib.json"));
        let sorted_lib_usage = utils::sort_kvmap_by_vsize::<String, String>(lib_usage.clone());
        if let Err(e) = serde_json::ser::to_writer_pretty(ranks_writer, &sorted_lib_usage)  {
            error!("Saving to json failed, {}", e);
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
        .arg(Arg::new("MaxClone").long("max").takes_value(true)
            .about("A number indicating the upper limit for this session"))
        .arg(Arg::new("Heavy").long("heavy")
            .takes_value(false).required(false)
            .about("Do execute mine_api_usage()"))
        .arg(Arg::new("CodeQLCreateDB").long("codeql-db-create")
            .takes_value(false).required(false)
            .about("Call CodeQL CLI to create database"))
        .get_matches()
}
