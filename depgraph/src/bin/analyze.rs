/// This program will use Soufflé to do analysis based on collected facts,
/// then added new founded edges (or possible nodes) onto the base graph.
/// Currently graphviz dot is used, may use other visualization tools
/// in the future.


use depgraph::utils::utils;
use depgraph::jar_class_map::MvnModule;
use depgraph::pomdep::PomGraph;
use depgraph::dl_relation as dlrel;
use depgraph::utils::err;
use clap::{App, ArgMatches, Arg, crate_authors, crate_version};
use std::path::Path;
use depgraph::dot_graph::DotStyle;
use std::io::{BufWriter, Write, stdout};
use std::fs::File;
use std::process::{Command, Stdio, exit};
use std::str::from_utf8;
use log::{warn,error};
use depgraph::dl_relation::SimpleLibPair;


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

    let dl_prog_path = Path::new(args.value_of("DL_PATH").unwrap());
    let added_pairs = run_souffle(mod_path, dl_prog_path);
    for p in &added_pairs {
        println!("{}", p);
    }
    let ss = DotStyle::create_with_all("box", "rounded", "Avenir", "14", 2);
    pom_graph.write_dot_preamble(&mut o_writer, &ss);
    pom_graph.to_dot_nodes(&mut o_writer, &ss);
    pom_graph.to_dot_edges(&mut o_writer, &ss);
    add_edge_to_graph(&pom_graph, &added_pairs, &mut o_writer);
    write!(o_writer, "}}\n").unwrap();
}

fn add_edge_to_graph<W: Write>(exist_graph: &PomGraph, pairs: &Vec<SimpleLibPair>, out: &mut W) {
    for p in pairs {
        let src_node = format!("{}:{}:jar", p.group_x(), p.artifact_x());
        let dst_node = format!("{}:{}:jar", p.group_y(), p.artifact_y());

        let attr_str = match p.kind() {
            dlrel::RelKind::SameGroupDataFlowLib => format!("[style=\"dashed\",color=\"blue\"]"),
            dlrel::RelKind::PomDepDataFlowLLib => format!("[style=\"dashed\",color=\"brown\"]"),
            dlrel::RelKind::PomDepDataFlowRLib => format!("[style=\"dashed\",color=\"darkorange\"]"),
            _ => String::new()
        };
        write!(out, "{:spaces$}\"{}\" -> \"{}\"{}\n", "", src_node, dst_node, attr_str, spaces = 2).unwrap();
    }

}

fn run_souffle(mod_path: &Path, program: &Path)
               -> Vec<dlrel::SimpleLibPair> {
    // souffle-orig -F "$MOD_PATH/.facts"  "${DL_PROGRAM_DIR}/def.dl" -D "$DL_OUT_DIR"
    let facts_dir = mod_path.join("./.facts");
    if !facts_dir.exists() {
        error!("{}", err::ErrorKind::PathNotExist(String::from(facts_dir.to_str().unwrap())));
        return vec!()
    }
    let dl_out_dir = mod_path.join("./dl-output");
    if !dl_out_dir.exists() {
        std::fs::create_dir_all(&dl_out_dir).unwrap();
    }
    match Command::new("souffle-orig")
        .arg("-F").arg(facts_dir)// facts dir
        .arg(program.to_str().unwrap_or_default()) // program
        .arg("-D").arg(&dl_out_dir) // sepcify output
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()).output() {
        Ok(c) => {
            if !c.status.success() {
                error!("{}: {}", err::ErrorKind::ExtCommandFailure(String::from("souffle-orig")),
                       std::str::from_utf8(&c.stderr).unwrap());
            }
            let mut all_pairs = Vec::new();
            for rel in dlrel::REL_KINDS.iter() {
                let kind = dlrel::RelKind::from_str(rel);
                let mut out_path = dl_out_dir.join(rel);
                out_path.set_extension("csv");
                match dlrel::read_simple_lib_pair(&out_path, kind) {
                    Ok(mut v) => all_pairs.append(&mut v),
                    Err(e) => error!("{}", e)
                }
            }
            all_pairs
        },
        Err(e) => {
            error!("{}: {}", err::ErrorKind::CallToExtCommandErr(String::from("souffle-orig")), e);
            vec!()
        }
    }
    /*
    let out_facts_path = dl_out_dir.join(rel);
    match dlrel::read_simple_lib_pair(&out_facts_path) {
        Err(e) => {error!("Error of type:", e.kind())},
        Ok(pairs) => {}
    }
     */
}

fn handle_args() -> ArgMatches {
    App::new("Analyzer and Visualizer: Run Souffle and visualize results")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("PATH").short('i').takes_value(true).required(true)
            .about("Path to the module"))
        .arg(Arg::new("DL_PATH").long("dl").takes_value(true).required(true)
            .about("Path to the Datalog program"))
        .arg(Arg::new("OutFile").short('o')
            .takes_value(true)
            .about("Specify output filename, otherwise print to stdout"))
        .get_matches()
}

