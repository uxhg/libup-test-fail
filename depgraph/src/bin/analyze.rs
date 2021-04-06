/// This program will use Soufflé to do analysis based on collected facts,
/// then added new founded edges (or possible nodes) onto the base graph.
/// Currently graphviz dot is used, may use other visualization tools
/// in the future.


use depgraph::utils::utils;
use depgraph::jar_class_map::MvnModule;
use depgraph::pomdep::PomGraph;
use clap::{App, ArgMatches, Arg, crate_authors, crate_version};
use std::path::Path;
use depgraph::dot_graph::DotStyle;
use std::io::{BufWriter, Write, stdout};
use std::fs::File;
use std::process::{Command, Stdio};
use std::str::from_utf8;
use log::warn;


fn main() {
    utils::init_log();
    let args = handle_args();
    let mod_path = Path::new(args.value_of("PATH").unwrap());

    let json_path = PomGraph::generate_dep_json(&mod_path).unwrap();
    let pom_graph = PomGraph::read_from_json(json_path).unwrap();

    let mut o_writer: BufWriter<Box<dyn Write>> = BufWriter::new(match args.value_of("OutFile") {
        Some(x) => Box::new(File::create(Path::new(x)).unwrap()),
        None => Box::new(stdout())
    });

    let ss = DotStyle::create_with_all("box", "rounded", "Avenir", "14", 2);

    pom_graph.write_dot_preamble(&mut o_writer, &ss);
    pom_graph.to_dot_nodes(&mut o_writer, &ss);
    pom_graph.to_dot_edges(&mut o_writer, &ss);

}

fn run_souffle(mod_path: &Path, program: &Path) -> Option<Vec<String>> {
    // souffle-orig -F "$MOD_PATH/.facts"  "${DL_PROGRAM_DIR}/def.dl" -D "$DL_OUT_DIR"
    let facts_dir = mod_path.join("./facts");
    let dl_out_dir = mod_path.join("./dl-output");
    let souffle_cmd = Command::new("souffle-orig")
        .arg("-F").arg(facts_dir)// facts dir
        .arg(program.to_str().unwrap_or_default()) // program
        .arg("-D").arg(dl_out_dir) // sepcify output
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()).output().ok()?;
    if !souffle_cmd.status.success() {
        warn!("Errors occurred when running souffle: {}", std::str::from_utf8(&souffle_cmd.stderr).unwrap());
    }
    match from_utf8(&souffle_cmd.stdout) {
        Ok(out) => Some(out.split('\n').map(|x| x.to_owned()).collect::<Vec<String>>()),
        Err(e) => {
            warn!("Cannot parse souffle output. {}", e);
            None
        }
    }

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

