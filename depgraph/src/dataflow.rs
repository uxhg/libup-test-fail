use std::collections::HashSet;
use std::io::Write;

use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct Flow {
    #[serde(rename = "source")]
    src_method: String,
    #[serde(rename = "lib1")]
    src_class: String,
    #[serde(rename = "sink")]
    dst_method: String,
    #[serde(rename = "lib2")]
    dst_class: String,
}

impl Flow {
    pub fn src_method(&self) -> &str {
        &self.src_method
    }
    pub fn src_class(&self) -> &str {
        &self.src_class
    }
    pub fn dst_method(&self) -> &str {
        &self.dst_method
    }
    pub fn dst_class(&self) -> &str {
        &self.dst_class
    }
}


#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct CFlow {
    s: String,
    d: String,
}

impl CFlow {
    pub fn from_flow(f: Flow) -> CFlow {
        CFlow { s: f.src_class, d: f.dst_class }
    }
    pub fn from_tuple(a: String, b: String) -> CFlow {
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
    class_flows: Vec<CFlow>,
}

impl FlowGraph {
    pub fn from_csv(file_path: String) -> Result<FlowGraph, csv::Error> {
        let mut g: Vec<Flow> = Vec::new();
        let mut reader = csv::Reader::from_path(file_path)?;
        for row in reader.deserialize() {
            let record: Flow = row?;
            // println!("{:?}", &record);
            g.push(record);
        }
        let h = FlowGraph::extract_class_flow(&g);
        Ok(FlowGraph { flows: g, class_flows: h })
    }

    pub fn from_csv_with_filter(file_path: String, include: &Option<clap::Values>, exclude: &Option<clap::Values>) -> Result<FlowGraph, csv::Error> {
        // only filter dst_class, since currently codeql results already ensure src_class is not
        // from client
        let mut g: Vec<Flow> = Vec::new();
        let mut reader = csv::Reader::from_path(file_path)?;
        for row in reader.deserialize() {
            let record: Flow = row?;
            /*
            if vec!(record.src_class(), record.dst_class()).iter().any(|x| x.contains(exclude)) {
                // skip if at least one of nodes is to be excluded
                continue;
            }
            if vec!(record.src_class(), record.dst_class()).iter().all(|x| !x.contains(include)) {
                // skip if neither of nodes is to be included
                continue;
            } */
            // if !record.src_class().contains(include) && !record.dst_class().contains(exclude) { continue; }
            // println!("{:?}", &record);
            g.push(record);
        }

        struct Satisfy<F> where F: FnMut(&Flow) -> bool {
            pub pred: F,
            // pub filter: &'a str
        }

        /*
        fn satisfy(x: &str) -> Box<dyn Fn(&Flow) -> bool + '_ > {
            Box::new(|f| vec!(f.src_class(), f.dst_class()).iter().any(|y| y.contains(x)))
        }

        fn not_satisfy(x: &str) -> Box<dyn Fn(&Flow) -> bool + '_> {
            Box::new(|f| !vec!(f.src_class(), f.dst_class()).iter().any(|y| y.contains(x)))
        }*/

        let mut g_filter = match include {
            Some(x) => {
                // let satisfy_inc =vec!(record.src_class(), record.dst_class()).iter().any(|x| x.contains(include));
                let vals: Vec<&str> = x.clone().collect();
                let satisfy = Satisfy {
                    pred: |f| vals.iter().any(|s| vec!(f.src_class(), f.dst_class()).iter().any(|y| y.starts_with(s))),
                };
                g.into_iter().filter(satisfy.pred).collect()
            }
            None => g
        };
        g_filter = match exclude {
            Some(x) => {
                // let satisfy_exc =vec!(record.src_class(), record.dst_class()).iter().any(|x| x.contains(exclude));
                let vals: Vec<&str> = x.clone().collect();
                let not_satisfy = Satisfy {
                    pred: |f| !vals.iter().any(|s| vec!(f.src_class(), f.dst_class()).iter().any(|y| y.starts_with(s))),
                    // pred: |f| !vec!(f.src_class(), f.dst_class()).iter().any(|y| y.contains(x)),
                };
                g_filter.into_iter().filter(not_satisfy.pred).collect()
            }
            None => g_filter
        };

        let h = FlowGraph::extract_class_flow(&g_filter);
        Ok(FlowGraph { flows: g_filter, class_flows: h })
    }
    fn extract_class_flow(flows: &Vec<Flow>) -> Vec<CFlow> {
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

    /*
    pub fn all_nodes(&self) -> HashSet<&str> {
        let f_vec = self.get_class_flows();
        f_vec.iter().map(|x| x.s()).collect::<HashSet<&str>>().union(
            &f_vec.iter().map(|x| x.d()).collect::<HashSet<&str>>()).collect()
    }
     */

    pub fn all_nodes_sorted(&self) -> Vec<&str> {
        let f_vec = self.get_class_flows();
        let mut result: Vec<&str> = f_vec.iter().map(|x| x.s()).collect();
        result.append(&mut f_vec.iter().map(|x| x.d()).collect::<Vec<&str>>());
        result.sort_unstable();
        result.dedup();
        result
    }

    pub fn to_datalog<W: Write>(&self, out: &mut W) {
        for x in &self.flows {
            write!(out, "{}\t{}\t{}\t{}\n", x.src_method(), x.src_class(), x.dst_method(),
                   x.dst_class()).unwrap();
        }
    }
}
