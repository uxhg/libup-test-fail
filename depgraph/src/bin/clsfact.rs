// Generate facts: Jar(artifact) contain classes

use std::fs::File;
use std::io::{BufWriter, stdout, Write};
use std::path::Path;

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};

use depgraph::jar_class_map::MvnModule;
use depgraph::utils::utils;
use depgraph::mvn_reactor::MvnReactor;

fn main() {
    utils::init_log();
    let matches = handle_args();
    let repo_path = matches.value_of("INPUT").unwrap();
    let project_name = Path::new(repo_path).file_name().unwrap().to_str().unwrap();

    // write
    let mut o_writer: BufWriter<Box<dyn Write>> = BufWriter::new(
        match matches.value_of("OutFile") {
            Some(x) => Box::new(File::create(Path::new(x)).unwrap()),
            None => Box::new(stdout())
        });
    let mvn_proj = {
        let mut tmp = MvnReactor::new(project_name, repo_path);
        tmp.populate_mods_jar_class_map();
        tmp
    };
    mvn_proj.jar_class_map_to_facts(&mut o_writer);
    // let mvn_mod = {
    //     let mut tmp = MvnModule::new(project_name, repo_path);
    //     tmp.populate_jar_map();
    //     tmp // make it immutable after populate_jar_map()
    // };
    // mvn_mod.jar_class_map_to_facts(&mut o_writer);

    if matches.is_present("cslicer") {
        let mod_path = Path::new(repo_path);
        match utils::create_cslicer_config(mod_path, &mut BufWriter::new(File::create(mod_path.join("cslicer.properties")).unwrap())) {
            Err(e) => println!("{}", e),
            Ok(_) => ()
        }
    }
}

fn handle_args() -> ArgMatches {
    App::new("Facts extractor: JarContainClass")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("INPUT").short('i').long("input").takes_value(true)
            .about("Path to the module")
            .required(true))
        .arg(Arg::new("OutFile").short('o')
            .takes_value(true)
            .about("Specify output filename, otherwise print to stdout"))
        .arg(Arg::new("cslicer").long("cslicer").takes_value(false).required(false)
            .about("Generate config and invoke CSlicer"))
        .get_matches()
}

