use crate::jar_class_map::MvnModule;
use std::process::{Command, Stdio};
use std::path::Path;
use log::{error,warn,info};
use std::str;
use std::io::Write;

pub struct MvnReactor {
    name: String,
    path: String,
    modules: Vec<MvnModule>
}

impl MvnReactor {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn modules(&self) -> &Vec<MvnModule> {
        &self.modules
    }

    pub fn get_modules_mutable(&mut self) -> &mut Vec<MvnModule> {
        &mut self.modules
    }

    pub fn new(name: &str, path: &str) -> MvnReactor {
        let mods: Vec<MvnModule> = match Command::new("mvn").arg("exec:exec")
            .arg("-Dexec.executable=pwd").arg("-q").current_dir(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()).output() {
            Err(e) => {
                error!("Errors when invoking maven");
                vec![]
            },

            Ok(c) => {
                // maven seems not mixed stdout and stderr into stdout
                if c.stderr.len() != 0 {
                    warn!("Errors in mvn listing sub-module paths: {}",
                          str::from_utf8(&c.stderr).unwrap())
                }
                info!("List of modules:\n{:?}", str::from_utf8(&c.stdout).unwrap().split('\n').collect::<Vec<&str>>());
                str::from_utf8(&c.stdout).unwrap().split("\n").filter(|x| x.contains("/"))
                    .map(|x| MvnModule::new(x.rsplit_once("/").unwrap().0, x))
                    .collect()
                //c.stdout.split('\n').collect()
            }
        };

        MvnReactor{
            name: name.to_string(),
            path: path.to_string(),
            modules: mods
        }
    }


    pub fn populate_mods_jar_class_map(&mut self) {
        for x in &mut self.modules {
            x.populate_jar_map()
        }
    }

    pub fn jar_class_map_to_facts<W: Write>(&self, o_writer: &mut W) {
        for m in self.modules() {
            m.jar_class_map_to_facts(o_writer);
        }
    }

    pub fn mvn_pkg_skiptests(&self) -> bool {
        match Command::new("mvn").arg("clean")
            .arg("package").arg("-DskipTests")
            .current_dir(self.path()).stderr(Stdio::piped()).output() {
            Ok(mvn_cmd) => {
                if mvn_cmd.stderr.len() != 0 {
                    warn!("Stderr in [mvn package]: {}", std::str::from_utf8(&mvn_cmd.stderr).unwrap());
                    false
                } else {
                    info!("[mvn package] External command finished.");
                    true
                }
            }
            Err(e) => {
                error!("Errors when trying to run [mvn package]: {}", e);
                false
            },
        }
    }
    /*
    args: list = shlex.split('mvn exec:exec -Dexec.executable="pwd" -q')
    completed: subprocess.CompletedProcess = subprocess.run(args, cwd=root_path, shell=False, capture_output=True)
    try:
    completed.check_returncode()
    except subprocess.CalledProcessError:
    return []
    else:
    ret = [y for x in completed.stdout.decode().split("\n") if len(y := x.rstrip("\n").strip()) > 0]
    logger.info(f"The list of modules: {ret}")
    return ret
    */
}