use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use dirs;
use env_logger;
use git2::{Repository, RepositoryOpenFlags};
use log::{error, info, warn};
use url::Url;

use crate::utils::err;
use crate::utils::err::ErrorKind;


pub fn init_log() {
    let env = env_logger::Env::default()
        .filter_or("RUST_LOG", "warn")
        .write_style_or("LOG_STYLE", "auto");
    env_logger::Builder::from_env(env).init();
    // env_logger::init_from_env(env);
}

/// Search repo from a given path
pub fn get_repo(mod_path: &Path) -> Option<Repository> {
    let ceil_path = dirs::home_dir().unwrap().join("Projects");
    match Repository::open_ext(mod_path, RepositoryOpenFlags::empty(), vec![ceil_path]) {
        Ok(repo) => Some(repo),
        Err(_e) => {
            warn!("{} is not a git repo", mod_path.to_str().unwrap());
            return None
        }
    }

}

pub fn get_repo_head(repo: &Repository) -> Result<String, err::Error> {
    let c = repo.head()?.peel_to_commit()?;
    Ok(c.id().to_string())
}


/// Write a CSlicer configuration file according given a module path
///
/// We use utils::get_repo() to search upwards to the root path of the repo
/// and hardcoded mod_path/target/temp/unpack as classRoot.
/// This method is here, since CSlicer is used to generate facts about reference relations
/// between classes.
pub fn create_cslicer_config<W: Write>(mod_path: &Path, out: &mut W) -> Result<(), err::Error> {
    let repo = get_repo(mod_path);
    match repo {
        None => {
            Err(err::Error::new(ErrorKind::Others(format!("Cannot find a repo from {}, \
            thus a valid CSlicer config cannot be generated.", mod_path.to_str().unwrap()))))
        },
        Some(r) => {
            write!(out, "repoPath = {}\n", r.path().to_str().unwrap())?;
            // write!(out, "classRoot = {}\n",
            //        mod_path.join("target/temp/unpack").to_str().unwrap())?;
            write!(out, "classRoot = {}\n", mod_path.to_str().unwrap())?;
            return match get_repo_head(&r) {
                Err(e) => Err(e.into()),
                Ok(cmt) => {
                    write!(out, "endCommit = {}\n", cmt)?;
                    Ok(())
                }
            }
        }
    }
}


pub fn load_json<P: AsRef<Path>>(file_path: P) -> BufReader<File> {
    let file_path_str = file_path.as_ref().to_str().expect("Cannot convert path to str");
    info!("Read JSON @ {}", &file_path_str);
    let f = File::open(file_path.as_ref()).expect(&format!("Cannot open file @ {}", &file_path_str));
    BufReader::new(f)
}

pub fn clone_remote(url: &str, local_path: &Path) -> Option<Repository> {
    let mut clone_to_path = PathBuf::from(local_path);
    match Url::parse(url) {
        Err(e) => {
            error!("Cannot parse {}, skip. Errors: {}", url, e);
            return None
        },
        Ok(u) => {
            if let Some(s) = u.path_segments() {
                clone_to_path.extend(s);
            } else {
                warn!("Cannot split path for {}, skip", url);
                return None
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
