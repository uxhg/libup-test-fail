use depgraph::dot_graph;
use depgraph::dataflow::FlowGraph;

use std::io::{BufWriter, Write, stdout};
use std::fs::File;
use std::path::Path;

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};

fn main() {
    let args = handle_args();
    let df_csv = args.value_of("INPUT").unwrap();
    let exclude_filter = args.values_of("exclude");
    let include_filter = args.values_of("include");

    let mut o_writer: BufWriter<Box<dyn Write>> = BufWriter::new(match args.value_of("OutFile") {
        Some (x) => Box::new(File::create(Path::new(x)).unwrap()),
        None => Box::new(stdout())
    });

    //let dp_graph = match filter_class {
    let dp_graph = FlowGraph::from_csv_with_filter(String::from(df_csv), &include_filter, &exclude_filter).unwrap();
    //   None => FlowGraph::from_csv(String::from(df_csv)).unwrap(),
    //};
    // let dp_graph = FlowGraph::from_csv(String::from(df_csv)).unwrap();
    let dot_edges = dot_graph::DotGraph::from_flow_graph(&dp_graph);

    dot_graph::render_to(&dot_edges, &mut o_writer);
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
        .get_matches()
}
