use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// The structure is used for parsing SARIF file generated by CodeQL
#[derive(Serialize, Deserialize)]
pub struct Sarif {
    #[serde(rename = "$schema")]
    schema: String,
    version: String,
    #[serde(rename = "runs")]
    runs: Vec<SarifRuns>
}

#[derive(Serialize, Deserialize)]
pub struct SarifRuns {
    tool: serde_json::Value,
    artifacts: serde_json::Value,
    results: Vec<SarifResults>,
    #[serde(rename = "columnKind")]
    column_kind: String,
    properties: serde_json::Value
}


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifResults {
    rule_id: String,
    rule_index: u8,
    rule: serde_json::Value,
    message: ResultMsg,
    locations: Vec<SarifLocation>,
    code_flows: Vec<CodeFlow>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct ResultMsg {
    text: String
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifLocation {
    physical_location: Vec<SarifPhysicalLocation>
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifPhysicalLocation {
    artifact_location: serde_json::Value,
    region: SarifLocationRegion
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifLocationRegion {
    start_line: u32,
    start_column: u32,
    end_column: u32
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeFlow {
    thread_flows: Vec<ThreadFlow>
}

#[derive(Serialize, Deserialize)]
pub struct ThreadFlow {
    locations: Vec<SarifLocation>
}
