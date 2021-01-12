use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct Flow {
    #[serde(rename = "source")]
    src_method: String,
    #[serde(rename = "lib1")]
    src_artifact: String,
    #[serde(rename = "sink")]
    dst_method: String,
    #[serde(rename = "lib2")]
    dst_artifact: String
}


#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct LibFlow {
    s: String,
    d: String
}

impl LibFlow {
    pub fn from_flow(f: Flow) -> LibFlow {
        LibFlow { s: f.src_artifact, d: f.dst_artifact }
    }
    pub fn from_tuple(a: String, b:String) -> LibFlow {
        LibFlow { s: a, d: b }
    }
}

pub struct FlowGraph {
    flows: Vec<Flow>,
    lib_flows: Vec<LibFlow>
}

impl FlowGraph {
    pub fn from_csv(file_path: String) -> Result<FlowGraph, csv::Error> {
        let mut g: Vec<Flow> =  Vec::new();
        let mut reader = csv::Reader::from_path(file_path)?;
        for row in reader.deserialize() {
            let record: Flow = row?;
            // println!("{:?}", &record);
            g.push(record);
        }
        let h = FlowGraph::extract_lib_flow(&g);
        Ok(FlowGraph {flows: g, lib_flows: h })
    }

    fn extract_lib_flow(flows: &Vec<Flow>) -> Vec<LibFlow> {
        let mut s: HashSet<(&String, &String)> = HashSet::new();
        for f in flows {
            s.insert((&f.src_artifact, &f.dst_artifact));
        }
        let mut lf: Vec<LibFlow> = Vec::new();
        for edge in s {
            lf.push(LibFlow::from_tuple(String::from(edge.0), String::from(edge.1)))
        }
        lf
    }

    pub fn get_lib_flows(&self) -> &Vec<LibFlow> {
        &self.lib_flows
    }
}
