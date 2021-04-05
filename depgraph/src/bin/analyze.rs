/// This program will use Soufflé to do analysis based on collected facts,
/// then added new founded edges (or possible nodes) onto the base graph.
/// Currently graphviz dot is used, may use other visualization tools
/// in the future.


use depgraph::utils::utils;
use depgraph::jar_class_map::MvnModule;
use depgraph::pomdep::PomGraph;
use clap::{App, ArgMatches, Arg, crate_authors, crate_version};
use std::path::Path;


fn main() {
    utils::init_log();
    let args = handle_args();
    let mod_path = Path::new(args.value_of("PATH").unwrap());

    let json_path = PomGraph::generate_dep_json(&mod_path).unwrap();
    let pom_graph = PomGraph::read_from_json(json_path).unwrap();
    pom_graph.to_dot()

}

fn handle_args() -> ArgMatches {
    App::new("Analyzer and Visualizer: Run Souffle and visualize results")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("PATH").short('i').takes_value(true).required(true)
            .about("Path to the module"))
        .arg(Arg::new("OutFile").short('o')
            .takes_value(true)
            .about("Specify output filename, otherwise print to stdout"))
        .arg(Arg::new("fmt").long("fmt").takes_value(true).required(true)
            .about("Specify output format, currently impl: souffle"))
        .get_matches()
}

