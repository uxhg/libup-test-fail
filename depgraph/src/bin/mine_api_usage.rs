use std::fs;
/// This program mine API usages from client facts.
/// 1. Produce dependency facts extracted from POM
/// 2. also Origins of depgraph (multiple modules if it is a reactor)
/// 3. Call CSlicer for generating usual FuncCall/Contain facts
/// 4.
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use clap::{App, Arg, ArgMatches, crate_authors, crate_version};
use log::{error, info, warn};

use depgraph::jar_class_map::MvnModule;
use depgraph::mvn_reactor::MvnReactor;
use depgraph::pomdep::write_pom_dep;
use depgraph::utils::utils;

fn main() {
    utils::init_log();
    let matches = handle_args();
    let repo_path = Path::new(matches.value_of("INPUT").unwrap());
    let project_name = repo_path.file_name().unwrap().to_str().unwrap();
    let out_dir = Path::new(matches.value_of("OUTPUT").unwrap_or("output"));
    if !out_dir.exists() {
        fs::create_dir(out_dir).unwrap();
    }

    let mut pom_dep_writer = BufWriter::new(File::create(out_dir.join("PomDep.facts")).unwrap());
    let pom_graph = write_pom_dep(repo_path, "souffle", "aggregate", &mut pom_dep_writer);

    let mut origins_writer = BufWriter::new(File::create(out_dir.join("PomDepOrigin.facts")).unwrap());
    for x in pom_graph.find_origins_coord(){
        write!(origins_writer, "{}\n", x.to_dl_string()).expect("Failed to write Origins.facts");
    }
    let mut direct_dep_w = BufWriter::new(File::create(out_dir.join("DirectDep.facts")).unwrap());
    for x in pom_graph.find_direct_dep() {
        write!(direct_dep_w, "{}\n", x.to_dl_string()).expect("Failed to write DirectDep.facts");
    }


    let cslicer_cfg_path = repo_path.join(format!("{}.properties", project_name));
    // let cslicer_cfg_path = Path::new("cslicer.properties");
    match utils::create_cslicer_config(repo_path, &mut BufWriter::new(File::create(&cslicer_cfg_path).unwrap())) {
        Err(e) => panic!("Abort because of creation of CSlicer configuration failed. Because: {}", e),
        Ok(_) => info!("CSlicer configuration file created successfully @ {}", cslicer_cfg_path.to_str().unwrap_or_default())
    }

    if matches.is_present("Build") {
        match matches.value_of("Build-Script") {
            Some(v) => todo!(),
            None => {
                let mut mvn_proj = MvnReactor::new(project_name, repo_path.to_str().unwrap());
                if !mvn_proj.mvn_pkg_skiptests() {
                    panic!("Abort because of mvn package failure.")
                }
            }
        }
    }

    if matches.is_present("CSlicer-Run") {
        let cslicer_jar_path = matches.value_of("CSlicer").unwrap();
        match Command::new("java").arg("-jar").arg(cslicer_jar_path)
            .arg("-e").arg("dl").arg("-ext").arg("dep").arg("-c").arg(&cslicer_cfg_path)
            .current_dir(repo_path).stderr(Stdio::piped()).output() {
            Ok(cslicer_cmd) => {
                if cslicer_cmd.stderr.len() != 0 {
                    warn!("Errors in running CSlicer: {}", std::str::from_utf8(&cslicer_cmd.stderr).unwrap());
                }
                info!("Facts generated inside {}", repo_path.join(".facts").to_str().unwrap())
            }
            Err(e) => error!("Errors when trying to run [mvn package]: {}", e)
        }
    }

    // any call to populate_mods_jar_class_map() should be after call to CSlier,
    // otherwise it will generate facts on all dependency classes
    if matches.is_present("JarClassMap") {
        let mut jar_contain_class_writer = BufWriter::new(File::create(out_dir.join("ContainClass.facts")).unwrap());
        let mvn_proj = {
            let mut tmp = MvnReactor::new(project_name, repo_path.to_str().unwrap());
            tmp.populate_mods_jar_class_map();
            tmp
        };
        mvn_proj.jar_class_map_to_facts(&mut jar_contain_class_writer);
    }
}

fn handle_args() -> ArgMatches {
    App::new("Mine library API Usages: from client facts")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("INPUT").short('i').long("input").takes_value(true)
            .about("Path to the module")
            .required(true))
        .arg(Arg::new("CSlicer").short('j').long("cslicer-jar")
            .takes_value(true).required(false)
            .about("Specify path to CSlicer JAR"))
        .arg(Arg::new("Build").short('b').long("build")
            .takes_value(false).required(false)
            .about("Build with mvn package -DskipTests at module path"))
        .arg(Arg::new("JarClassMap").long("jar-class-map")
            .takes_value(false).required(false)
            .about("Produce Jar-Class-Map facts"))
        .arg(Arg::new("CSlicer-Run").long("cslicer-run")
            .takes_value(false).required(false).requires("CSlicer")
            .about("Run CSlicer to produce facts"))
        .arg(Arg::new("Build-Script").long("build-script").takes_value(true)
            .requires("Build")
            .about("Use specified script for building"))
        .arg(Arg::new("OUTPUT").short('o').long("out-dir").takes_value(true)
            .about("Path to the output directory"))
        .get_matches()
}
