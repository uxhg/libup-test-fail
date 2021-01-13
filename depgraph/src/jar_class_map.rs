use std::process::{Command, Stdio};
use std::collections::{HashMap, hash_map::RandomState};
use std::path::Path;
use std::error::Error;
use log::{info, warn, error};
use crate::pomdep::MvnCoord;
use walkdir::WalkDir;
use std::fs;
use std::io::{BufReader, BufRead};
use std::hash::Hash;

/*
pub struct JarArtifact {
    mvn_coord: MvnCoord,
    class_list: Vec<String>
}

impl JarArtifact {
    pub fn mvn_coord(&self) -> &MvnCoord {
        &self.mvn_coord
    }
    pub fn class_list(&self) -> &Vec<String> {
        &self.class_list
    }
} */


pub struct Jar{
    name: String,
    artifacts: HashMap<MvnCoord, Vec<String>>
}

impl Jar {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn artifacts(&self) -> &HashMap<MvnCoord, Vec<String>, RandomState> {
        &self.artifacts
    }

    fn new(jar_path: &Path) -> Jar {
        Jar {
            name: String::from(jar_path.file_stem().unwrap().to_str().unwrap()),
            artifacts: Jar::extract_jar(jar_path).unwrap(),
        }
    }

    fn read_pom_properties(file_path: &str) -> Option<MvnCoord> {
        let f = match fs::File::open(file_path) {
            Ok(file) => file,
            Err(e) => {return None}
        };
        let mut coord: MvnCoord = MvnCoord::default();
        for line in BufReader::new(f).lines(){
            let l = line.ok()?;
            if !l.starts_with(r"#") {
                let x: Vec<&str> = l.split("=").collect();
                match x[0] {
                    "version" => coord.set_version_id(String::from(x[1])),
                    "groupId" => coord.set_group_id(String::from(x[1])),
                    "artifactId" => coord.set_artifact_id(String::from(x[1])),
                    _ => warn!("{}: Unexpected contents {}", file_path, x[0])
                }
            }
        }
        Some(coord)
    }

    fn extract_jar(jar_path: &Path) -> Option<HashMap<MvnCoord, Vec<String>>> {
        if !jar_path.is_file() {
            error!("{}: is not a file", jar_path.to_str().unwrap());
            return None;
        }
        let x = jar_path.file_stem().unwrap().to_str().unwrap();
        let dir = match jar_path.parent() {
            Some(d)=> d,
            None => {
                error!("{}: cannot get parent of this path", jar_path.to_str().unwrap());
                return None;
            }
        };
        let extracted_path = dir.join(x);
        if extracted_path.is_dir() {
            warn!("{}: exists", &extracted_path.to_str().unwrap());
        } else {
            fs::create_dir(&extracted_path);
        }
        let extract_cmd = Command::new("jar").arg("xf")
            .arg(jar_path.to_str().unwrap())
            .current_dir(&extracted_path.to_str().unwrap())
            .stderr(Stdio::piped()).output().ok()?;
        if !extract_cmd.status.success() {
            warn!("Errors in jar extraction: {}", std::str::from_utf8(&extract_cmd.stderr).unwrap());
        }
        let mut found_coords: Vec<MvnCoord> = vec!();
        for entry in WalkDir::new(&extracted_path.join("META-INF")) {
            //.filter_entry(|e| e.file_name().to_str().unwrap() == "pom.properties") {
            let e = entry.ok()?;
            if e.file_name().to_str().unwrap() == "pom.properties" {
                let pom_path = e.path().to_str().unwrap();
                info!("Read {}", pom_path);
                let coord = Jar::read_pom_properties(pom_path).unwrap();
                // let group_path = coord.group_id().replace(".", "/").replace("-", "_");
                found_coords.push(coord);
            }
        }
        let classes: Vec<String> = WalkDir::new(&extracted_path).into_iter()
            .filter_entry(|e| e.path().ends_with(".class"))
            .map(|x| String::from(x.unwrap().path().to_str().unwrap())).collect();
        println!("{}", classes.len());
        let mut results: HashMap<MvnCoord, Vec<String>> = HashMap::new();
        for clazz in classes {
            println!("{}", clazz)
        }
        Some(results)
    }

}

pub struct MvnModule {
    name: String,
    jar_map: HashMap<String, Jar>
}

impl MvnModule {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn jar_map(&self) -> &HashMap<String, Jar> {
        &self.jar_map
    }

    pub fn new(module_name: &str, module_path: &str) -> MvnModule {
        return MvnModule {
            name: String::from(module_name),
            jar_map: MvnModule::copy_dep(module_path).unwrap()
        }
    }

    pub fn copy_dep(root_path: &str) -> Result<HashMap<String, Jar>, Box<dyn Error>> { //-> Result<>{
        let dep_jar_path= "target/temp";
        let mvn_cp_dep = Command::new("mvn").arg("clean").
            arg("dependency:copy-dependencies").
            arg(format!("-DoutputDirectory={}", dep_jar_path))
            .current_dir(root_path).stderr(Stdio::piped()).output()?;
        if mvn_cp_dep.stderr.len() != 0 {
            warn!("Errors in copy-dep: {}", std::str::from_utf8(&mvn_cp_dep.stderr).unwrap());
        }
        let temp_path = Path::new(root_path).join(dep_jar_path);
        let mut jar_map: HashMap<String, Jar> = HashMap::new();
        info!("Working @ {}", &temp_path.to_str().unwrap());
        for entry in WalkDir::new(temp_path.to_str().unwrap()) {
            let e = entry.unwrap();
            if e.path().is_file() && e.path().extension().unwrap().to_str().unwrap() == "jar"{
                let jar_name = String::from(e.file_name().to_str().unwrap());
                info!("Add {}", &jar_name);
                jar_map.insert(jar_name, Jar::new(e.path()));
            }
            // println!("{}", entry.unwrap().path().extension().unwrap().to_str().unwrap());
        }
        info!("In total {} jars added", jar_map.len());
        Ok(jar_map)
    }
}

/*
struct ArtifactClassMap {
    root: String,
    multi_module: bool,
    module_list: Vec<MvnModule>
}

impl JarClassMap {
    pub fn new(root_path: &str) -> JarClassMap {
        return JarClassMap {
            root: String::from(root_path),
            multi_module: false,
            module_list: vec!()
        }
    }

    pub fn extract_jar(&self) {
        for m in &self.module_list {
            let m_path = Path::new(&self.root).join(&m.name);

        }
    }


}
*/
// "mvn clean dependency:copy-dependencies -DoutputDirectory=target/temp"