// Generate facts: Data dependency
use std::fs::File;
use std::io::{BufWriter, stdout, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};
use log::warn;

use depgraph::pomdep::PomGraph;
use depgraph::utils::utils;

fn main() {
    utils::init_log();
    let args = handle_args();

    let mod_path = Path::new(args.value_of("PATH").unwrap());
    let json_path = generate_dep_json(&mod_path).unwrap();

    let mut o_writer: BufWriter<Box<dyn Write>> = BufWriter::new(match args.value_of("OutFile") {
        Some(x) => Box::new(File::create(Path::new(x)).unwrap()),
        None => Box::new(stdout())
    });
    let out_fmt = args.value_of("fmt").unwrap();

    let pom_graph = PomGraph::read_from_json(json_path).unwrap();
    match out_fmt {
        // "dot" => write_dot(&pom_graph, &mut o_writer),
        "souffle" => write_souffle(&pom_graph, &mut o_writer),
        _ => warn!("'{}' is unsupported output format, use one of {{souffle}}", out_fmt)
    };
}

fn write_souffle<W: Write>(pom_graph: &PomGraph, out: &mut W) {
    pom_graph.to_datalog(out);
}

/*
fn write_dot<W: Write>() {
    // not implemented yet
}*/

fn generate_dep_json(path: &Path) -> Option<PathBuf> {
    let depgraph_cmd = Command::new("mvn").current_dir(path).arg("-DgraphFormat=JSON")
        .arg("-DshowDuplicates").arg("-DshowConflicts")
        .arg("com.github.ferstl:depgraph-maven-plugin:graph")
        .stderr(Stdio::piped()).output().ok()?;
    let plugin_url = "https://github.com/ferstl/depgraph-maven-plugin";
    if depgraph_cmd.stderr.len() != 0 {
        warn!("Errors in depgraph-maven-plugin JSON generation: {}\nRefer to {}",
              std::str::from_utf8(&depgraph_cmd.stderr).unwrap(), plugin_url);
    }

    let json_path = path.join("target/dependency-graph.json");
    if !json_path.is_file() {
        warn!("{} was not generated", json_path.to_str().unwrap());
        ()
    }
    Some(json_path)
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

