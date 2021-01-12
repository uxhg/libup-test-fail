use std::process::{Command, Stdio};
use std::path::Path;
use log::{info, warn};
use env_logger::Env;
use crate::pomdep::MvnCoord;

struct Jar {
    mvn_coord: MvnCoord,
    class_list: Vec<String>
}


impl Jar {

}

struct MvnModule {
    name: String,
    jar_list: Vec<Jar>
}

struct JarClassMap {
    root: Path,
    multi_module: bool,
    module_list: Vec<MvnModule>
}

impl JarClassMap {
    pub fn copy_dep(&self) -> Result<>{
        let mvn_cp_dep = Command::new("mvn").arg("clean").
            arg("dependency:copy-dependencies").arg("-DoutputDirectory=target/temp")
            .current_dir(&self.root).stderr(Stdio::piped()).output();

    }
}
// "mvn clean dependency:copy-dependencies -DoutputDirectory=target/temp"