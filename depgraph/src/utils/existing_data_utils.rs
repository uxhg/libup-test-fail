use serde::{Deserialize, Serialize};
use std::path::Path;
use url::Url;
use crate::utils::utils;
use log::error;


#[derive(Deserialize, Debug)]
pub struct RepoAtVer {
    name: String,
    sha: String,
    url: String,
    tag: String,
}

impl RepoAtVer {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn sha(&self) -> &str {
        &self.sha
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn batch_create_from_json<P: AsRef<Path>>(file_path: P) -> Vec<RepoAtVer> {
        let reader = utils::load_json(file_path.as_ref());
        serde_json::from_reader::<_, Vec<RepoAtVer>>(reader)
            .expect("Failed to deserialize from JSON to a list of repos.")
    }

}


