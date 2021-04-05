// Generate facts: Data dependency
use std::fs::File;
use std::io::{BufWriter, stdout, Write};
use std::path::Path;

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};
use log::warn;

use depgraph::pomdep::PomGraph;
use depgraph::utils::utils;

fn main() {
    utils::init_log();
    let args = handle_args();

    let mod_path = Path::new(args.value_of("PATH").unwrap());

    let mut o_writer: BufWriter<Box<dyn Write>> = BufWriter::new(match args.value_of("OutFile") {
        Some(x) => Box::new(File::create(Path::new(x)).unwrap()),
        None => Box::new(stdout())
    });
    let out_fmt = args.value_of("fmt").unwrap();

    get_pom_deps(mod_path, out_fmt, &mut o_writer)
}


fn get_pom_deps<W: Write>(mod_path: &Path, out_fmt: &str, o_writer: &mut BufWriter<W>) {
    let json_path = PomGraph::generate_dep_json(&mod_path).unwrap();
    let pom_graph = PomGraph::read_from_json(json_path).unwrap();
    match out_fmt {
        "dot" => write_dot(&pom_graph, o_writer),
        "souffle" => write_souffle(&pom_graph, o_writer),
        _ => warn!("'{}' is unsupported output format, use one of {{souffle}}", out_fmt)
    };
}

/// Generate Datalog facts in souffle dialects
fn write_souffle<W: Write>(pom_graph: &PomGraph, out: &mut W) {
    pom_graph.to_datalog(out);
}

/// Generate dot
fn write_dot<W: Write>(pom_graph: &PomGraph, out: &mut W) {
    pom_graph.to_dot(out);
}


fn handle_args() -> ArgMatches {
    App::new("Fact extractor: produce facts for pom dependencies")
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

