use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use log::{error, info, warn};

use crate::mvn_reactor::MvnReactor;
use crate::pomdep::write_pom_dep;
use crate::utils::err;
use crate::utils::utils;

pub fn mine_api_usage(repo_path: &Path, out_dir: &Path, build_flag: bool, build_script: Option<&str>,
                      cslicer_path: Option<&str>, gen_jar_class_map: bool) -> Result<(bool, bool, bool), err::Error> {
    let mut suc_flag = (false, false, false);
    let project_name = repo_path.file_name().unwrap().to_str().unwrap();
    info!("Start generating PomDep, PomDepOrigin and DirectDep facts");
    let mut pom_dep_writer = BufWriter::new(File::create(out_dir.join("PomDep.facts"))?);
    match write_pom_dep(repo_path, "souffle", "aggregate", &mut pom_dep_writer) {
        Some(g) => {
            let mut origins_writer = BufWriter::new(File::create(out_dir.join("PomDepOrigin.facts"))?);
            for x in g.find_origins_coord() {
                write!(origins_writer, "{}\n", x.to_dl_string()).expect("Failed to write Origins.facts");
            }
            let mut direct_dep_w = BufWriter::new(File::create(out_dir.join("DirectDep.facts"))?);
            for x in g.find_direct_dep() {
                write!(direct_dep_w, "{}\n", x.to_dl_string()).expect("Failed to write DirectDep.facts");
            }
            suc_flag.0 = true;
        }
        None => {
            warn!("Skip PomDep/PomDepOrigin/DirectDep facts because dependency-graph.json was not generated.");
        }
    }

    if build_flag { // -b
        match build_script {
            Some(v) => todo!(),
            None => {
                let mvn_proj = MvnReactor::new(project_name, repo_path.to_str()
                    .ok_or(err::Error::new(err::ErrorKind::Others(format!("Cannot convert repo_path {:?} to str", repo_path))))?);
                if !mvn_proj.mvn_pkg_skiptests() {
                    error!("Cannot build, mvn package failures, early termination.");
                    return Ok(suc_flag);
                }
            }
        }
    }

    if let Some(cslicer_jar_path) = cslicer_path { // --cslicer-run
        let cslicer_cfg_path = repo_path.join(format!("{}.properties", project_name));
        // let cslicer_cfg_path = Path::new("cslicer.properties");
        match utils::create_cslicer_config(repo_path, &mut BufWriter::new(File::create(&cslicer_cfg_path).unwrap())) {
            Err(e) => panic!("Abort because of creation of CSlicer configuration failed. Because: {}", e),
            Ok(_) => info!("CSlicer configuration file created successfully @ {}", cslicer_cfg_path.to_str().unwrap_or_default())
        }

        match Command::new("java").arg("-jar").arg(cslicer_jar_path)
            .arg("-e").arg("dl").arg("-ext").arg("dep").arg("-c").arg(&cslicer_cfg_path)
            .current_dir(repo_path).stderr(Stdio::piped()).output() {
            Ok(cslicer_cmd) => {
                if cslicer_cmd.stderr.len() != 0 {
                    warn!("Errors in running CSlicer: {}", std::str::from_utf8(&cslicer_cmd.stderr)?);
                } else {
                    info!("Facts generated inside {}", repo_path.join(".facts").to_str()
                        .ok_or(err::Error::new(err::ErrorKind::Others(format!("Cannot convert fact path {:?} to str", repo_path.join("facts")))))?);
                    suc_flag.1 = true;
                }
            }
            Err(e) => error!("Errors when trying to run [mvn package]: {}", e)
        }
    }

    // any call to populate_mods_jar_class_map() should be after call to CSlier,
    // otherwise it will generate facts on all dependency classes
    if gen_jar_class_map { // --jar-class-map
        let mut jar_contain_class_writer = BufWriter::new(File::create(out_dir.join("ContainClass.facts"))?);
        let mvn_proj = {
            let mut tmp = MvnReactor::new(project_name, repo_path.to_str().unwrap());
            tmp.populate_mods_jar_class_map();
            tmp
        };
        mvn_proj.jar_class_map_to_facts(&mut jar_contain_class_writer);
        suc_flag.2 = true;
    }
    Ok(suc_flag)
}
