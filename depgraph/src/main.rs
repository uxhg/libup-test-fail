use std::collections::{HashSet};

use log::{info, warn};

use depgraph::dataflow::FlowGraph;
use depgraph::jar_class_map::MvnModule;
use depgraph::pomdep::PomGraph;
use depgraph::pomdep::MvnCoord;
use depgraph::utils::utils;


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

fn add_lib_edges(dp_graph: FlowGraph, pom_graph: PomGraph, mvn_mod: MvnModule)
    -> HashSet<(String, String)> {
    let mut added_edges: HashSet<(String, String)> = HashSet::new();
    for f in dp_graph.get_lib_flows() {
        // println!("{:?}", f);
        if f.s().contains("com.github.dockerjava") || f.d().contains("com.github.dockerjava"){
            info!("Skip client");
            continue
        }
        let src_artifact = search_artifact(f.s(), &mvn_mod);
        match src_artifact {
            Some(s) =>  {
                let dst_artifact = search_artifact(f.d(), &mvn_mod);
                match dst_artifact {
                    Some(d) => {
                        let src_id = pom_graph.get_node_id(s);
                        let dst_id = pom_graph.get_node_id(d);
                        if src_id.is_none() || dst_id.is_none() {
                            warn!("Skip {} --> {}", s, d);
                        } else {
                            added_edges.insert((src_id.unwrap(), dst_id.unwrap()));
                        }
                    },
                    None => warn!("Cannot find artifact name for {}", f.d())
                }
            },
            None => warn!("Cannot find artifact name for {}", f.s())
        }
    }
    added_edges
}

fn main() {
    utils::init_log();

    let graph = PomGraph::read_from_json("data/docker-java-pom.json").unwrap();
    // println!("{}", graph.graph_name());

    let local_dep = MvnModule::new(
        "docker-java",
        "/home/wuxh/Projects/lib-conflict/cases/docker-java");


    let dp_graph = FlowGraph:: from_csv(String::from("data/docker-java-result.csv")).unwrap();

    let added_edges = add_lib_edges(dp_graph, graph, local_dep);
    for e in added_edges {
        println!("  \"{}\" -> \"{}\"[color=\"blue\"]", e.0, e.1);
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

