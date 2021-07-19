use std::collections::HashMap;
use std::path::Path;

use log::{error,warn};
use serde::{Deserialize, Serialize};

use crate::utils::utils;

/// The structure is used for parsing SARIF file generated by CodeQL
#[derive(Serialize, Deserialize)]
pub struct Sarif {
    #[serde(rename = "$schema")]
    schema: String,
    version: String,
    #[serde(rename = "runs")]
    runs: Vec<SarifRuns>,
}

impl Sarif {
    pub fn schema(&self) -> &str {
        &self.schema
    }
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn runs(&self) -> &Vec<SarifRuns> {
        &self.runs
    }
}

#[derive(Serialize, Deserialize)]
pub struct SarifRuns {
    tool: serde_json::Value,
    artifacts: serde_json::Value,
    results: Vec<SarifResult>,
    #[serde(rename = "columnKind")]
    column_kind: String,
    properties: serde_json::Value,
}

impl SarifRuns {
    pub fn results(&self) -> &Vec<SarifResult> {
        &self.results
    }
}


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifResult {
    rule_id: String,
    rule_index: u8,
    rule: serde_json::Value,
    message: ResultMsg,
    locations: Vec<SarifLocation>,
    code_flows: Vec<CodeFlow>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

impl SarifResult {
    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }
    pub fn rule_index(&self) -> u8 {
        self.rule_index
    }
    pub fn message(&self) -> &ResultMsg {
        &self.message
    }
    pub fn locations(&self) -> &Vec<SarifLocation> {
        &self.locations
    }
    pub fn code_flows(&self) -> &Vec<CodeFlow> {
        &self.code_flows
    }

    pub fn get_nth_loc_region(&self, n: usize) -> Option<&SarifLocationRegion> {
        if n >= self.locations().len() {
            warn!("{} is bigger than number of locations, index starts from 0", n);
            None
        }
        Some(self.locations()[n].physical_location().region())
    }

    pub fn get_nth_loc_artifact_loc(&self, n:usize) -> Option<&SarifArtifactLocation> {
        if n >= self.locations().len() {
            warn!("{} is bigger than number of locations, index starts from 0", n);
            None
        }
        Some(self.locations()[n].physical_location().artifact_location())
    }

    pub fn is_same_msg_and_region(&self, other: &SarifResult) -> bool {
        return self.message() == other.message()
            && self.get_nth_loc_artifact_loc(0) == other.get_nth_loc_artifact_loc(0)
            && self.get_nth_loc_region(0) == other.get_nth_loc_region(0)
    }
}


#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct ResultMsg {
    text: String,
}

impl ResultMsg {
    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifLocationMap {
    location: SarifLocation,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifLocation {
    physical_location: SarifPhysicalLocation,
    message: Option<ResultMsg>,
}

impl SarifLocation {
    pub fn physical_location(&self) -> &SarifPhysicalLocation {
        &self.physical_location
    }
    pub fn message(&self) -> &Option<ResultMsg> {
        &self.message
    }
    pub fn message_to_string(&self) -> Option<String> {
        match &self.message {
            Some(s) => Some(String::from(s.text())),
            None => None
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifPhysicalLocation {
    artifact_location: SarifArtifactLocation,
    region: SarifLocationRegion,
}

impl SarifPhysicalLocation {
    pub fn region(&self) -> &SarifLocationRegion {
        &self.region
    }
    pub fn artifact_location(&self) -> &SarifArtifactLocation {
        &self.artifact_location
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SarifLocationRegion {
    start_line: u32,
    start_column: u32,
    end_column: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SarifArtifactLocation {
    uri: String,
    uri_base_id: String,
    index: u32
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeFlow {
    thread_flows: Vec<ThreadFlow>,
}

impl CodeFlow {
    pub fn thread_flows(&self) -> &Vec<ThreadFlow> {
        &self.thread_flows
    }
}

#[derive(Serialize, Deserialize)]
pub struct ThreadFlow {
    locations: Vec<SarifLocationMap>,
}

impl ThreadFlow {
    pub fn locations(&self) -> &Vec<SarifLocationMap> {
        &self.locations
    }

    pub fn get_msg_seq(&self) -> Vec<Option<String>> {
        self.locations.iter()
            .map(|x| x.location.message_to_string())
            .collect()
    }
}

/// Parse a SARIF file
pub fn read_from_json<P: AsRef<Path>>(file_path: P) -> Option<Sarif> {
    let reader = utils::load_json(file_path.as_ref());
    match serde_json::from_reader::<_, Sarif>(reader) {
        Err(e) => {
            error!("Failed to deserialize from a Sarif file: {}", e);
            None
        }
        Ok(g) => Some(g)
    }
}

pub fn merge_paths(multiple_sarif: Vec<Sarif>) {
    let merged_flows: HashMap<String, Vec<String>> = HashMap::new();
    for s in multiple_sarif {
        for run in s.runs() {
            for each_result in run.results() {
                let target_api = each_result.message().text();
                let pos = each_result.get_nth_loc_region(0);
                for code_flow in each_result.code_flows() {
                    for tflow in code_flow.thread_flows() {
                        for msg in tflow.get_msg_seq() {
                            println!("{}", msg.unwrap_or_default());
                        }
                    }
                }
            }
        }
    }
}