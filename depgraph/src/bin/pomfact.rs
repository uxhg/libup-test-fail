// Generate facts: Data dependency
use std::fs::File;
use std::io::{BufWriter, stdout, Write};
use std::path::Path;

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};

use depgraph::pomdep;
use depgraph::utils::utils;

fn main() {
    utils::init_log();
    let args = handle_args();

    let mod_path = Path::new(args.value_of("PATH").unwrap());

    let mut o_writer: BufWriter<Box<dyn Write>> = BufWriter::new(match args.value_of("OutFile") {
        Some(x) => Box::new(File::create(Path::new(x)).unwrap()),
        None => Box::new(stdout())
    });
    let out_fmt = args.value_of("Format").unwrap();
    let goal = args.value_of("DepGraphGoal").unwrap_or_default();

    pomdep::write_pom_dep(mod_path, out_fmt, goal, &mut o_writer).unwrap();
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
        .arg(Arg::new("DepGraphGoal").short('g').takes_value(true)
            .about("Specify a goal, default to graph"))
        .arg(Arg::new("Format").long("fmt").takes_value(true).default_value("souffle")
            .about("Specify output format, default: souffle, currently impl: souffle"))
        .get_matches()
}

