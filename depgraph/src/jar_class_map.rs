use std::collections::{hash_map::RandomState, HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use log::{error, info, warn, debug};
use walkdir::WalkDir;

use crate::pomdep::MvnCoord;

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


pub struct Jar {
    name: String,
    artifacts: HashMap<MvnCoord, Vec<String>>,
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
            artifacts: match Jar::extract_jar(jar_path) {
                Some(a) => a,
                None => {
                    error!("Extraction failed for {}", jar_path.to_str().unwrap());
                    panic!();
                }
            },
        }
    }

    fn read_pom_properties(file_path: &str) -> Option<MvnCoord> {
        let f = match fs::File::open(file_path) {
            Ok(file) => file,
            Err(e) => {
                error!("Cannot open {}: {}", file_path, e.to_string());
                return None;
            }
        };
        let mut coord: MvnCoord = MvnCoord::default();
        for line in BufReader::new(f).lines() {
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

    fn match_artifact(info: &HashMap<String, String>, coord: &mut MvnCoord) {
        let (mut gid, mut aid, mut vid) = (None, None, None);
        let default_str = String::from("");
        let vids = vec!("Implementation-Version", "Bundle-Version").into_iter().filter_map(|x| info.get(x)).collect::<Vec<&String>>();
        if !vids.is_empty() {
            vid = Some(vids[0]);
        }
        // let mut vid = info.get("Implementation-Version").unwrap_or("")
        let impl_t = info.get("Implementation-Title").unwrap_or(&default_str);
        let impl_v = info.get("Implementation-Vendor").unwrap_or(&default_str);
        let impl_vid = info.get("Implementation-Vendor-Id").unwrap_or(&default_str);
        let ext_name = info.get("Extension-Name").unwrap_or(&default_str);
        if (impl_t == "hamcrest-library") && (impl_v == "hamcrest.org") {
            gid = Some("org.hamcrest");
            aid = Some("hamcrest-library");
        } else if (impl_t == "Objenesis") && (impl_vid == "org.objenesis") {
            gid = Some("obj.objenesis");
            aid = Some("objenesis");
        } else if (ext_name == "javax.persistence") && (impl_vid == "javax.persistence") {
            gid = Some("javax.persistence");
            aid = Some("persistence-api");
        } else if (info.get("Automatic-Module-Name").unwrap_or(&default_str) == "org.bouncycastle.provider")
            && (impl_vid == "org.bouncycastle") {
            gid = Some("org.bouncycastle");
            aid = Some("bcprov-jdk15on");
        } else if info.get("Bundle-SymbolicName").unwrap_or(&default_str) == "org.mockito.mockito-core" {
            gid = Some("org.mockito");
            aid = Some("mockito-core");
        }
        warn!("Match {} => {}:{}", impl_t, gid.unwrap_or("n/a"), aid.unwrap_or("n/a"));
        if gid.is_some() {
            coord.set_group_id(gid.unwrap().to_string());
        }
        if aid.is_some() {
            coord.set_artifact_id(aid.unwrap().to_string());
        }
        if vid.is_some() {
            coord.set_version_id(vid.unwrap().to_string());
        }
    }

    fn read_manifest(file_path: &str) -> Option<MvnCoord> {
        let f = match fs::File::open(file_path) {
            Ok(file) => file,
            Err(e) => {
                error!("Cannot open {}: {}", file_path, e.to_string());
                return None; }
        };
        let mut coord: MvnCoord = MvnCoord::default();
        // let x = BufReader::new(f).lines().for_each(|x| x.unwrap_or(String::from("")).split(":").collect());
        let mut info: HashMap<String, String> = HashMap::new();
        for line in BufReader::new(f).lines() {
            let l = line.ok()?;
            if !l.starts_with(r"#") && !l.starts_with(" ") && l.contains(":") {
                let x: Vec<&str> = l.split(":").collect();
                if x[0].len() == 0 {
                    continue;
                }
                info.insert(x[0].to_string(), String::from(x[1].trim_start_matches(" ")));
            }
        }
        Jar::match_artifact(&info, &mut coord);
        Some(coord)
    }

    fn match_coord<'a>(candidates: &'a HashSet<MvnCoord>, class_path: &str) -> Option<&'a MvnCoord> {
        // let prefix = candidates.into_iter().
        let path_elements = class_path.split("/").map(|x| x.to_string())
            .collect::<HashSet<String>>();
        // let c: HashSet<Vec<String>> = candidates.iter().map(|x| x.build_id_list()).collect();
        let mut max: usize = 0;
        let mut chosen = None;
        for c in candidates {
            let c_set: HashSet<String> = c.build_id_list().into_iter().collect();
            let int = c_set.intersection(&path_elements).collect::<HashSet<&String>>();
            if int.len() >= max {
                max = int.len();
                chosen = Some(c);
            }
        }
        chosen
    }

    fn extract_jar(jar_path: &Path) -> Option<HashMap<MvnCoord, Vec<String>>> {
        if !jar_path.is_file() {
            error!("{}: is not a file", jar_path.to_str().unwrap());
            return None;
        }
        let jar_name = jar_path.file_stem().unwrap().to_str().unwrap();
        let dir = match jar_path.parent() {
            Some(d) => d.join("unpack"),
            None => {
                error!("{}: cannot get parent of this path", jar_path.to_str().unwrap());
                return None;
            }
        };
        let extracted_path = dir.join(jar_name);
        if extracted_path.is_dir() {
            info!("{}: exists, and existing files will be used", &extracted_path.to_str().unwrap());
        } else {
            info!("Make dir: {}", &extracted_path.to_str().unwrap());
            fs::create_dir_all(&extracted_path).unwrap();
            let extract_cmd = Command::new("jar").arg("xf")
                .arg(jar_path.to_str().unwrap())
                .current_dir(&extracted_path.to_str().unwrap())
                .stderr(Stdio::piped()).output().ok()?;
            if !extract_cmd.status.success() {
                warn!("Errors in jar extraction: {}", std::str::from_utf8(&extract_cmd.stderr).unwrap());
            }
        }
        let mut found_coords: HashSet<MvnCoord> = HashSet::new();
        let meta_inf_dir = &extracted_path.join("META-INF");
        if !meta_inf_dir.is_dir() {
            warn!("No META-INF in {}", jar_name);
            found_coords.insert(MvnCoord::new("", jar_name, ""));
        } else {
            for entry in WalkDir::new(meta_inf_dir) {
                let e = entry.unwrap();
                if e.file_name().to_str().unwrap() == "pom.properties" {
                    let pom_path = e.path().to_str().unwrap();
                    info!("Read {}", pom_path);
                    let coord: MvnCoord = Jar::read_pom_properties(pom_path).unwrap();
                    // let group_path = coord.group_id().replace(".", "/").replace("-", "_");
                    found_coords.insert(coord);
                }
            }
            if found_coords.len() == 0 {
                warn!("[{}] No pom.properties, search MANIFEST.MF", jar_name);
                for entry in WalkDir::new(meta_inf_dir).into_iter()
                    .filter(|e| e.as_ref().unwrap().file_name().to_str().unwrap() == "MANIFEST.MF") {
                    let e = entry.ok()?;
                    let manifest_path = e.path().to_str().unwrap();
                    info!("Read {}", manifest_path);
                    match Jar::read_manifest(manifest_path) {
                        Some(x) => {
                            if x.is_all_set() {
                                found_coords.insert(x);
                            }
                        }
                        None => ()
                    };
                    // let group_path = coord.group_id().replace(".", "/").replace("-", "_");
                }
                if found_coords.len() == 0 {
                    warn!("No MANIFEST.MF found in {}", jar_name);
                }
            }
        }
        // NOTE: this filter by ending could include a directory in the form of xxx.class/
        let classes: Vec<String> = WalkDir::new(&extracted_path).into_iter()
            .map(|x| String::from(x.unwrap().path()
                .strip_prefix(&extracted_path).unwrap().to_str().unwrap()))
            .filter(|e| e.ends_with(".class")).collect();
        info!("{} classes in jar [{}]", classes.len(), &extracted_path.file_name().unwrap().to_str().unwrap());
        let mut results: HashMap<MvnCoord, Vec<String>> = HashMap::new();
        for clazz in classes {
            match Jar::match_coord(&found_coords, &clazz) {
                Some(c) => {
                    debug!("Check class: {}", clazz);
                    let class_name = String::from(clazz.replace("/", ".")
                        .strip_suffix(".class").unwrap());
                    if results.contains_key(c) {
                        results.get_mut(c).unwrap().push(class_name);
                    } else {
                        results.insert(c.clone(), vec!(class_name));
                    }
                    // let chosen = c;
                    // println!("{}\tbelongs to\t{}",clazz, chosen)
                }
                None => {
                    warn!("Cannot choose artifact for class: {}", clazz);
                }
            };
        }
        Some(results)
    }
}

pub struct MvnModule {
    name: String,
    jar_map: HashMap<String, Jar>,
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
            jar_map: MvnModule::copy_dep(module_path).unwrap(),
        };
    }

    pub fn copy_dep(root_path: &str) -> Result<HashMap<String, Jar>, Box<dyn Error>> { //-> Result<>{
        let dep_jar_path = "target/temp/";
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
        for entry in WalkDir::new(temp_path).into_iter()
            .filter_entry(|e| {
                let ext = e.path().extension();
                e.depth() == 0 || (ext.is_some() && ext.unwrap().to_str().unwrap() == "jar")
            }) {
            let e = entry.unwrap();
            info!("Read {}", e.path().to_str().unwrap());
            if e.path().is_file() && e.path().extension().unwrap().to_str().unwrap() == "jar" {
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
