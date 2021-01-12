use std::cmp::PartialEq;
use std::io::BufReader;
use std::fs::File;
use std::error::Error;
use std::path::Path;
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
