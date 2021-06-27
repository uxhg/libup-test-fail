use std::io::Write;
use std::path::Path;

use dirs;
use env_logger;
use git2::{Repository, RepositoryOpenFlags};
use log::warn;

use crate::utils::err;
use crate::utils::err::ErrorKind;
use std::collections::HashMap;

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

