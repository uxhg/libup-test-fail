use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use log::warn;
use serde::{Deserialize, Serialize};

use crate::dot_graph::DotStyle;

#[derive(PartialEq, Eq, Serialize, Deserialize, Hash, Debug)]
#[serde(rename_all = "lowercase")]
pub enum MvnPkgType {
    Jar,
    Pom,
    Bundle,
    Ejb,
    War,
    Ear,
    Rar,
    #[serde(rename = "maven-plugin")]
    MavenPlugin
}

impl fmt::Display for MvnPkgType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Hash, Debug)]
#[serde(rename_all = "lowercase")]
pub enum MvnScope {
    Compile,
    Provided,
    Runtime,
    Test,
    System,
    Import,
}

impl fmt::Display for MvnScope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Resolution {
    #[serde(rename = "INCLUDED")]
    Included,
    #[serde(rename = "OMITTED_FOR_DUPLICATE")]
    OmittedForDuplicate,
    #[serde(rename = "OMITTED_FOR_CONFLICT")]
    OmittedForConflict,
}

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MvnCoord {
    group_id: String,
    artifact_id: String,
    #[serde(rename = "version")]
    version_id: String,
}

impl MvnCoord {
    pub fn group_id(&self) -> &str {
        &self.group_id
    }
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
    pub fn set_group_id(&mut self, group_id: String) {
        self.group_id = group_id;
    }
    pub fn set_artifact_id(&mut self, artifact_id: String) {
        self.artifact_id = artifact_id;
    }
    pub fn set_version_id(&mut self, version_id: String) {
        self.version_id = version_id;
    }

    pub fn is_all_set(&self) -> bool {
        self.group_id.len() != 0 && self.artifact_id.len() != 0 && self.version_id.len() != 0
    }

    pub fn build_id_list(&self) -> Vec<String> {
        let gid = self.group_id.replace("-", "_");
        let aid = self.artifact_id.replace("-", "_");
        let mut coord_elements = gid.split(".").map(|x| x.to_string())
            .collect::<Vec<String>>();
        // let mut gid = ;
        coord_elements.append(&mut aid.split(".").map(|x| x.to_string())
            .collect::<Vec<String>>());
        coord_elements
    }

    pub fn new(g: &str, a: &str, v: &str) -> MvnCoord {
        MvnCoord {
            group_id: String::from(g),
            artifact_id: String::from(a),
            version_id: String::from(v),
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.group_id, self.artifact_id, self.version_id)
    }
    pub fn to_dl_string(&self) -> String {
        format!("{}\t{}\t{}", self.group_id, self.artifact_id, self.version_id)
    }
}

impl Default for MvnCoord {
    fn default() -> MvnCoord {
        MvnCoord {
            group_id: String::from(""),
            artifact_id: String::from(""),
            version_id: String::from(""),
        }
    }
}

impl fmt::Display for MvnCoord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "G: {}\tA:{}\tV:\t{}", self.group_id, self.artifact_id, self.version_id)
    }
}

/*
impl Clone for MvnCoord {
    fn clone(&self) -> MvnCoord {
        MvnCoord {
            group_id: String::from(self.group_id()),
            artifact_id: String::from(self.artifact_id()),
            version_id: String::from(self.version_id()),
        }
    }
}

 */

#[derive(PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
pub struct GraphNode {
    id: String,
    numeric_id: u32,
    #[serde(flatten)]
    mvn_coord: MvnCoord,
    optional: bool,
    classifiers: Option<Vec<String>>,
    scopes: Vec<MvnScope>,
    #[serde(rename = "types")]
    packaging: Vec<MvnPkgType>,
}

impl GraphNode {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn mvn_coord(&self) -> &MvnCoord {
        &self.mvn_coord
    }
    pub fn numeric_id(&self) -> u32 {
        self.numeric_id
    }
    pub fn optional(&self) -> bool {
        self.optional
    }
    pub fn classifiers(&self) -> &Option<Vec<String>> {
        &self.classifiers
    }
    pub fn scopes(&self) -> &Vec<MvnScope> {
        &self.scopes
    }
    pub fn packaging(&self) -> &Vec<MvnPkgType> {
        &self.packaging
    }

    /* This is unnecessary, just use id
    pub fn grp_art_pkg_label (&self) -> Vec<String> {
        self.packaging.iter().map(|x| format!(
            "{}:{}:{}", self.mvn_coord().group_id(), self.mvn_coord().artifact_id(), x)).collect()
    }*/
}

/// An edge describing a dependency from a package to another, as stated in pom
/// The structure intimidates the structure of JSON output of ferstl/depgraph maven plugin
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PomDepEdge {
    from: String,
    to: String,
    numeric_from: u32,
    numeric_to: u32,
    resolution: Resolution,
    version: Option<String>
}

impl PomDepEdge {
    pub fn from(&self) -> &str {
        &self.from
    }
    pub fn to(&self) -> &str {
        &self.to
    }
    pub fn numeric_from(&self) -> u32 {
        self.numeric_from
    }
    pub fn numeric_to(&self) -> u32 {
        self.numeric_to
    }
    pub fn resolution(&self) -> &Resolution {
        &self.resolution
    }
    pub fn version(&self) -> &Option<String> {
        &self.version
    }
}

/// A graph describing package-level dependencies as stated in pom.
/// The structure intimidates the structure of JSON output of ferstl/depgraph maven plugin
#[derive(Serialize, Deserialize)]
pub struct PomGraph {
    #[serde(rename = "graphName")]
    graph_name: String,
    artifacts: HashSet<GraphNode>,
    dependencies: Vec<PomDepEdge>,
}

impl PomGraph {
    pub fn graph_name(&self) -> &str {
        &self.graph_name
    }
    pub fn artifacts(&self) -> &HashSet<GraphNode> {
        &self.artifacts
    }
    pub fn dependencies(&self) -> &Vec<PomDepEdge> {
        &self.dependencies
    }


    /// Construct a PomGraph from a JSON file
    pub fn read_from_json<P: AsRef<Path>>(file_path: P) -> Result<PomGraph, Box<dyn Error>> {
        let f = File::open(file_path)?;
        let reader = BufReader::new(f);
        let g: PomGraph = serde_json::from_reader(reader).unwrap();
        Ok(g)
    }

    /// Build a hashmap of PomGraph.artifacts so that we have convenient access
    /// to all info when inspecting a dependency edge
    pub fn build_nodes_hashmap(&self) -> HashMap<u32, &GraphNode> {
        let mut m = HashMap::new();
        for x in &self.artifacts {
            m.insert(x.numeric_id, x);
        }
        m
    }

    /// Given a MvnCoord, find a most matched node in the graph
    pub fn get_node_id(&self, coord: &MvnCoord) -> Option<String> {
        for x in self.artifacts() {
            if x.mvn_coord() == coord {
                return Some(String::from(x.id()));
            }
        }
        warn!("{} not found in the list of artifacts, switch to fuzzy matching", coord);
        for x in self.artifacts() {
            if x.mvn_coord().artifact_id() == coord.artifact_id() {
                warn!("Fuzzy match: {} == {}", coord.artifact_id(), x.id());
                return Some(String::from(x.id()));
            }
        }
        warn!("{} not found even with fuzzy match, skip", coord);
        None
    }

    /// Use ferstl/depgraph maven plugin to generate pom dep in JSON
    /// # Arguments
    /// * `path` - A `&Path` to a maven module
    /// # Return
    /// of type `Option<PathBuf>`, the path to the generated JSON file if succeeded
    pub fn generate_dep_json(path: &Path) -> Option<PathBuf> {
        let depgraph_cmd = Command::new("mvn").current_dir(path).arg("-DgraphFormat=JSON")
            .arg("-DshowDuplicates").arg("-DshowConflicts")
            .arg("com.github.ferstl:depgraph-maven-plugin:graph")
            .stderr(Stdio::piped()).output().ok()?;
        let plugin_url = "https://github.com/ferstl/depgraph-maven-plugin";
        if depgraph_cmd.stderr.len() != 0 {
            warn!("Errors in depgraph-maven-plugin JSON generation: {}\nRefer to {}",
                  std::str::from_utf8(&depgraph_cmd.stderr).unwrap(), plugin_url);
        }

        let json_path = path.join("target/dependency-graph.json");
        match json_path.is_file() {
            true => Some(json_path),
            false => {warn!("{} was not generated", json_path.to_str().unwrap()); None}
        }
    }

    /// Output all PomGraph.dependencies as Datalog facts
    pub fn to_datalog<W: Write>(&self, out: &mut W) {
        let m = self.build_nodes_hashmap();
        for x in &self.dependencies {
            let from_coord = match m.get(&(x.numeric_from()+1)) { //.unwrap().mvn_coord();
                Some(v) => v.mvn_coord().to_dl_string(),
                None => {
                    warn!("Numeric id {} not found in artifacts set", x.numeric_from()+1);
                    MvnCoord::default().to_dl_string()
                }
            };
            let to_coord = match m.get(&(x.numeric_to() + 1)) { //.unwrap().mvn_coord();
                Some(v) => v.mvn_coord().to_dl_string(),
                None => {
                    warn!("Numeric id {} not found in artifacts set", x.numeric_to() + 1);
                    MvnCoord::default().to_dl_string()
                }
            };
            write!(out, "{}\t{}\t{}\n", from_coord, to_coord, x.resolution()).unwrap();
        }
    }


    /// Generate a graphviz dot file from PomGraph
    pub fn to_dot<W: Write>(&self, out: &mut W, ss: &DotStyle) {
        self.write_dot_preamble(out, ss);
        // main parts
        self.to_dot_nodes(out, ss);
        self.to_dot_edges(out, ss);

        // end
        write!(out, "}}\n").unwrap();
    }

    /// Write preamble parts of a digraph dot file
    pub fn write_dot_preamble<W: Write>(&self, out: &mut W, ss: &DotStyle) {
        write!(out, "digraph \"{}\" {{\n", &self.graph_name).unwrap();
        write!(out, "{:spaces$}{}\n", "", ss.node_style_decl(), spaces = ss.indent()).unwrap();
        write!(out, "{:spaces$}{}\n", "", ss.edge_style_decl(), spaces = ss.indent()).unwrap();
    }

    pub fn to_dot_nodes<W: Write>(&self, out: &mut W, ss: &DotStyle) {
        write!(out, "{:spaces$}// Node Definitions:\n", "", spaces = ss.indent()).unwrap();
        for n in &self.artifacts {
            let coord = n.mvn_coord();

            let mut scope_line = String::new();
            if n.scopes().len() > 0 && (n.scopes().len() != 1 || n.scopes()[0] != MvnScope::Compile) {
                scope_line = format!("<font point-size=\"10\"><br/>({})</font>",
                                     n.scopes().iter().map(|x| x.to_string()).collect::<Vec<String>>().join("/"));
            }
            write!(out, "{:spaces$}\"{}\"[label=<<font point-size=\"10\">{}</font><br/>{}<font point-size=\"10\"><br/>{}</font>{}>]\n",
                   "", n.id(), coord.group_id(), coord.artifact_id(), coord.version_id(), scope_line, spaces=ss.indent()).unwrap();
        }
    }

    pub fn to_dot_edges<W: Write>(&self, out: &mut W, ss: &DotStyle) {
        write!(out, "{:spaces$}// Edge Definitions:\n", "", spaces=ss.indent()).unwrap();
        for e in &self.dependencies {
            let attr_str = match e.resolution() {
                Resolution::Included => String::new(),
                Resolution::OmittedForDuplicate => String::from("[style=\"dotted\"]"),
                Resolution::OmittedForConflict => format!("[style=\"dashed\",color=\"red\",fontcolor=\"red\",label=\"{}\"]",
                                                          e.version().as_ref().unwrap_or(&String::from("unknown")))
            };
            write!(out, "{:spaces$}\"{}\" -> \"{}\"{}\n",
                   "", e.from(), e.to(), attr_str, spaces = ss.indent()).unwrap();
        }

    }

}
