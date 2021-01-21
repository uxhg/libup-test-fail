use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct Flow {
    #[serde(rename = "source")]
    src_method: String,
    #[serde(rename = "lib1")]
    src_class: String,
    #[serde(rename = "sink")]
    dst_method: String,
    #[serde(rename = "lib2")]
    dst_class: String
}


#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct CFlow {
    s: String,
    d: String
}

impl CFlow {
    pub fn from_flow(f: Flow) -> CFlow {
        CFlow { s: f.src_class, d: f.dst_class }
    }
    pub fn from_tuple(a: String, b:String) -> CFlow {
        CFlow { s: a, d: b }
    }
    pub fn s(&self) -> &str {
        &self.s
    }
    pub fn d(&self) -> &str {
        &self.d
    }
}

pub struct FlowGraph {
    flows: Vec<Flow>,
    class_flows: Vec<CFlow>
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
        Ok(FlowGraph {flows: g, class_flows: h })
    }

    fn extract_lib_flow(flows: &Vec<Flow>) -> Vec<CFlow> {
        let mut s: HashSet<(&String, &String)> = HashSet::new();
        for f in flows {
            s.insert((&f.src_class, &f.dst_class));
        }
        let mut lf: Vec<CFlow> = Vec::new();
        for edge in s {
            lf.push(CFlow::from_tuple(String::from(edge.0), String::from(edge.1)))
        }
        lf
    }

    pub fn get_class_flows(&self) -> &Vec<CFlow> {
        &self.class_flows
    }
}
