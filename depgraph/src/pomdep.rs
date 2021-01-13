use std::cmp::PartialEq;
use std::io::BufReader;
use std::fs::File;
use std::error::Error;
use std::path::Path;
use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum MvnDepType {
    Jar, Pom, Bundle
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum MvnScope {
    Compile, Provided, Runtime, Test, System, Import
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
enum Resolution {
    #[serde(rename = "INCLUDED")]
    Included,
    #[serde(rename = "OMITTED_FOR_DUPLICATE")]
    OmittedForDuplicate,
    #[serde(rename = "OMITTED_FOR_CONFLICT")]
    OmittedForConflict
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
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
}

impl Default for MvnCoord {
    fn default() -> MvnCoord {
        MvnCoord{
            group_id: String::from(""),
            artifact_id: String::from(""),
            version_id: String::from("")
        }
    }
}

impl fmt::Display for MvnCoord {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        write!(f, "G: {}\tA:{}\tV:\t{}", self.group_id, self.artifact_id, self.version_id)
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphNode {
    id: String,
    numeric_id: u32,
    #[serde(flatten)]
    mvn_coord: MvnCoord,
    optional : bool,
    classifiers: Option<Vec<String>>,
    scopes : Vec<MvnScope>,
    #[serde(rename = "types")]
    dep_types : Vec<MvnDepType>
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PomDepEdge {
    from: String,
    to: String,
    numeric_from: u32,
    numeric_to: u32,
    resolution: Resolution
}

#[derive(Serialize, Deserialize)]
pub struct PomGraph {
    #[serde(rename = "graphName")]
    graph_name: String,
    artifacts: Vec<GraphNode>,
    dependencies: Vec<PomDepEdge>
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
}