// Generate facts: Data dependency
use std::fs::File;
use std::io::{BufWriter, stdout, Write};
use std::path::Path;

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};
use log::warn;

use depgraph::dataflow::FlowGraph;
use depgraph::dot_graph;
use depgraph::utils::utils;

fn main() {
    utils::init_log();
    let args = handle_args();
    let df_csv = args.value_of("INPUT").unwrap();
    let exclude_filter = args.values_of("exclude");
    let include_filter = args.values_of("include");

    let mut o_writer: BufWriter<Box<dyn Write>> = BufWriter::new(match args.value_of("OutFile") {
        Some(x) => Box::new(File::create(Path::new(x)).unwrap()),
        None => Box::new(stdout())
    });

    //let dp_graph = match filter_class {
    let dp_graph = FlowGraph::from_csv_with_filter(String::from(df_csv), &include_filter, &exclude_filter).unwrap();
    //   None => FlowGraph::from_csv(String::from(df_csv)).unwrap(),
    //};
    // let dp_graph = FlowGraph::from_csv(String::from(df_csv)).unwrap();
    let out_fmt = args.value_of("fmt").unwrap();
    match out_fmt {
        "dot" => write_dot(&dp_graph, &mut o_writer),
        "souffle" => write_souffle(&dp_graph, &mut o_writer),
        _ => warn!("'{}' is unsupported output format, use one of {{souffle, dot}}", out_fmt)
    };
}

fn write_souffle<W: Write>(dp_graph: &FlowGraph, out: &mut W) {
    dp_graph.to_datalog(out);
}

fn write_dot<W: Write>(dp_graph: &FlowGraph, o_writer: &mut W) {
    let dot_edges = dot_graph::DotGraph::from_flow_graph(dp_graph);
    dot_graph::render_to(&dot_edges, o_writer);
}

fn handle_args() -> ArgMatches {
    App::new("Dot Exporter: produce dot graph representing relations between classes")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("INPUT").short('i').takes_value(true).required(true)
            .about("Path to the dataflow csv"))
        .arg(Arg::new("exclude").long("ex").takes_value(true).multiple(true)
            .about("Exclude classes whose names contain the string"))
        .arg(Arg::new("include").long("in").takes_value(true).multiple(true)
            .about("Include classes whose names contain the string. \
            If a is both included and excluded, it will be excluded."))
        .arg(Arg::new("OutFile").short('o')
            .takes_value(true)
            .about("Specify output filename, otherwise print to stdout"))
        .arg(Arg::new("fmt").long("fmt").takes_value(true).required(true)
            .about("Specify output format: souffle, dot"))
        .get_matches()
}

