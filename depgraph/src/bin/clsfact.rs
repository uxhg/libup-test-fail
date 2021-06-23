use std::error::Error;
// Generate facts: Jar(artifact) contain classes
use std::fs::File;
use std::io::{BufWriter, stdout, Write};
use std::path::Path;

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};
use log::warn;

use depgraph::jar_class_map::MvnModule;
use depgraph::utils::utils;

fn main() {
    utils::init_log();
    let matches = handle_args();
    let mod_path = matches.value_of("INPUT").unwrap();
    let mod_name = Path::new(mod_path).file_name().unwrap().to_str().unwrap();

    let local_dep = {
        let mut tmp = MvnModule::new(mod_name, mod_path);
        tmp.populate_jar_map();
        tmp // make it immutable after populate_jar_map()
    };
    let out_file = matches.value_of("OutFile");
    print_tuples(local_dep, out_file);

    if matches.is_present("cslicer") {
        let mod_path = Path::new(mod_path);
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

fn print_tuples(mvn_mod: MvnModule, out_file: Option<&str>) {
    let mut o_writer: BufWriter<Box<dyn Write>> = BufWriter::new(match out_file {
        Some(x) => Box::new(File::create(Path::new(x)).unwrap()),
        None => Box::new(stdout())
    });

    for j in mvn_mod.jar_map() {
        for (coord, clazz) in j.1.artifacts() {
            let row = format!("{}\t{}\t{}\t", coord.group_id(), coord.artifact_id(), coord.version_id());
            clazz.iter().for_each(|x| {
                write!(o_writer, "{}{}\n", row.clone(), x).unwrap();
            });
        }
    }
    o_writer.flush().unwrap();
}

