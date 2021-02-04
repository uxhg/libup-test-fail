use std::path::Path;

use env_logger::Env;
use git2::{Repository, RepositoryOpenFlags};
use log::warn;
use dirs;

pub fn init_log() {
    let env = Env::default()
        .filter_or("RUST_LOG", "warn")
        .write_style_or("LOG_STYLE", "always");
    env_logger::init_from_env(env);
}

pub fn get_repo(mod_path: &Path) -> Option<Repository> {
    let ceil_path = dirs::home_dir().unwrap().join("Projects");
    match Repository::open_ext(mod_path, RepositoryOpenFlags::empty(), vec![ceil_path]) {
        Ok(repo) => Some(repo),
        Err(_e) => {
            warn!("{} is not a git repo, thus a valid CSlicer config cannot be generated", mod_path.to_str().unwrap());
            return None
        }
    }

}

pub fn get_repo_head(repo: &Repository) -> Result<String, git2::Error> {
    let head = repo.head();
    match head {
        Ok(r) => {
            match r.peel_to_commit() {
                Ok(n) => Ok(String::from(n.id().to_string())),
                Err(e) => {warn!("Cannot get name of HEAD for repo."); Err(e)}
            }
        },
        Err(e) => {warn!("Cannot get current HEAD of the repo"); Err(e)}
    }
}


