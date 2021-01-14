// mod jar_class_map;
mod dataflow;
mod pomdep;
mod jar_class_map;
use log::{info, warn};
use env_logger::Env;
use std::collections::{HashSet, HashMap};
use crate::pomdep::MvnCoord;

use jar_class_map::MvnModule;
use serde::private::ser::constrain;

fn init_log() {
    let env = Env::default()
        .filter_or("RUST_LOG", "warn")
        .write_style_or("LOG_STYLE", "always");
    env_logger::init_from_env(env);
}

fn merge_from_ref(map: &mut HashMap<(), ()>, map_ref: &HashMap<(), ()>) {
    map.extend(map_ref.into_iter().map(|(k, v)| (k.clone(), v.clone())));
}

fn search_artifact<'a>(class_name: &str, mvn_module: &'a MvnModule) -> Option<&'a MvnCoord> {
    for j in mvn_module.jar_map() {
        for (coord, clazz) in j.1.artifacts(){
            let class_set: HashSet<String> = clazz.iter().map(|x| String::from(x)).collect();
            if class_set.contains(class_name) {
                return Some(coord);
            }
        }
    }
    None
}

fn main() {
    init_log();

    let graph = pomdep::PomGraph::read_from_json("data/docker-java-pom.json").unwrap();
    // println!("{}", graph.graph_name());

    let local_dep = MvnModule::new(
        "docker-java",
        "/home/wuxh/Projects/lib-conflict/cases/docker-java");


    let dp_graph = dataflow::FlowGraph:: from_csv(String::from("data/docker-java-result.csv")).unwrap();

    let mut added_edge_dot: HashSet<String> = HashSet::new();
    for f in dp_graph.get_lib_flows() {
    	// println!("{:?}", f);
        if f.s().contains("com.github.dockerjava") || f.d().contains("com.github.dockerjava"){
            info!("Skip client");
            continue
        }
        let src_artifact = search_artifact(f.s(), &local_dep);
        match src_artifact {
            Some(coord) =>  {
                let dst_artifact = search_artifact(f.d(), &local_dep);
                match dst_artifact {
                    Some(coord_b) => {
                        let src_coord = src_artifact.unwrap();
                        let dst_coord = dst_artifact.unwrap();
                        // println!("{} --> {}", src_coord, dst_coord);
                        let src_id = graph.get_node_id(src_coord);
                        let dst_id = graph.get_node_id(dst_coord);
                        if src_id.is_none() || dst_id.is_none() {
                            warn!("Skip {} --> {}", src_coord, dst_coord);
                        } else {
                            added_edge_dot.insert(format!("  \"{}\" -> \"{}\"[color=\"blue\"]", src_id.unwrap(), dst_id.unwrap()));
                        }
                    },
                    None => warn!("Cannot find artifact name for {}", f.d())
                }
            },
            None => warn!("Cannot find artifact name for {}", f.s())
        }
    }
    for e in added_edge_dot {
        println!("{}", e);
    }

    //"/home/wuxh/Projects/lib-conflict/cases/openscoring-codeql/openscoring-client");
    /*
    println!("{}", z.name());
    for j in z.jar_map() {
        println!("{}", j.0);
        for j_art in j.1.artifacts() {
            println!("{}", j_art.0);
            for clazz in j_art.1 {
                println!("{}", clazz)
            }
        }
    }
     */
}

