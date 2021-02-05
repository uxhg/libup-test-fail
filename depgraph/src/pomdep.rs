use std::cmp::PartialEq;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

use log::warn;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum MvnDepType {
    Jar,
    Pom,
    Bundle,
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum MvnScope {
    Compile,
    Provided,
    Runtime,
    Test,
    System,
    Import,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Resolution {
    #[serde(rename = "INCLUDED")]
    Included,
    #[serde(rename = "OMITTED_FOR_DUPLICATE")]
    OmittedForDuplicate,
    #[serde(rename = "OMITTED_FOR_CONFLICT")]
    OmittedForConflict,
}

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MvnCoord {
    group_id: String,
    artifact_id: String,
    #[serde(rename = "version")]
    version_id: String,
}

impl MvnCoord {
    pub fn group_id(&self) -> &str {
        &self.group_id
    }
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
    pub fn set_group_id(&mut self, group_id: String) {
        self.group_id = group_id;
    }
    pub fn set_artifact_id(&mut self, artifact_id: String) {
        self.artifact_id = artifact_id;
    }
    pub fn set_version_id(&mut self, version_id: String) {
        self.version_id = version_id;
    }

    pub fn is_all_set(&self) -> bool {
        self.group_id.len() != 0 && self.artifact_id.len() != 0 && self.version_id.len() != 0
    }

    pub fn build_id_list(&self) -> Vec<String> {
        let gid = self.group_id.replace("-", "_");
        let aid = self.artifact_id.replace("-", "_");
        let mut coord_elements = gid.split(".").map(|x| x.to_string())
            .collect::<Vec<String>>();
        // let mut gid = ;
        coord_elements.append(&mut aid.split(".").map(|x| x.to_string())
            .collect::<Vec<String>>());
        coord_elements
    }

    pub fn new(g: &str, a: &str, v: &str) -> MvnCoord {
        MvnCoord {
            group_id: String::from(g),
            artifact_id: String::from(a),
            version_id: String::from(v),
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.group_id, self.artifact_id, self.version_id)
    }
}

impl Default for MvnCoord {
    fn default() -> MvnCoord {
        MvnCoord {
            group_id: String::from(""),
            artifact_id: String::from(""),
            version_id: String::from(""),
        }
    }
}

impl fmt::Display for MvnCoord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "G: {}\tA:{}\tV:\t{}", self.group_id, self.artifact_id, self.version_id)
    }
}

/*
impl Clone for MvnCoord {
    fn clone(&self) -> MvnCoord {
        MvnCoord {
            group_id: String::from(self.group_id()),
            artifact_id: String::from(self.artifact_id()),
            version_id: String::from(self.version_id()),
        }
    }
}

 */

#[derive(PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNode {
    id: String,
    numeric_id: u32,
    #[serde(flatten)]
    mvn_coord: MvnCoord,
    optional: bool,
    classifiers: Option<Vec<String>>,
    scopes: Vec<MvnScope>,
    #[serde(rename = "types")]
    dep_types: Vec<MvnDepType>,
}

impl GraphNode {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn mvn_coord(&self) -> &MvnCoord {
        &self.mvn_coord
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PomDepEdge {
    from: String,
    to: String,
    numeric_from: u32,
    numeric_to: u32,
    resolution: Resolution,
}

impl PomDepEdge {
    pub fn from(&self) -> &str {
        &self.from
    }
    pub fn to(&self) -> &str {
        &self.to
    }
    pub fn numeric_from(&self) -> u32 {
        self.numeric_from
    }
    pub fn numeric_to(&self) -> u32 {
        self.numeric_to
    }
    pub fn resolution(&self) -> &Resolution {
        &self.resolution
    }
}

#[derive(Serialize, Deserialize)]
pub struct PomGraph {
    #[serde(rename = "graphName")]
    graph_name: String,
    artifacts: Vec<GraphNode>,
    dependencies: Vec<PomDepEdge>,
}

impl PomGraph {
    pub fn graph_name(&self) -> &str {
        &self.graph_name
    }
    pub fn artifacts(&self) -> &Vec<GraphNode> {
        &self.artifacts
    }
    pub fn dependencies(&self) -> &Vec<PomDepEdge> {
        &self.dependencies
    }

    pub fn read_from_json<P: AsRef<Path>>(file_path: P) -> Result<PomGraph, Box<dyn Error>> {
        let f = File::open(file_path)?;
        let reader = BufReader::new(f);
        let g: PomGraph = serde_json::from_reader(reader).unwrap();
        Ok(g)
    }

    pub fn get_node_id(&self, coord: &MvnCoord) -> Option<String> {
        for x in self.artifacts() {
            if x.mvn_coord() == coord {
                return Some(String::from(x.id()));
            }
        }
        warn!("{} not found in the list of artifacts, switch to fuzzy matching", coord);
        for x in self.artifacts() {
            if x.mvn_coord().artifact_id() == coord.artifact_id() {
                warn!("Fuzzy match: {} == {}", coord.artifact_id(), x.id());
                return Some(String::from(x.id()));
            }
        }
        warn!("{} not found even with fuzzy match, skip", coord);
        None
    }


    pub fn to_datalog<W: Write>(&self, out: &mut W) {
        for x in &self.dependencies {
            out.write(format!("{}\t{}\t{}\n", x.from(), x.to(), x.resolution()).as_bytes()).unwrap();
        }
    }
}
