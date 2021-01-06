use std::cmp::PartialEq;

#[derive(PartialEq, Eq)]
enum MvnDepType {
    Jar, Pom
}

enum MvnScope {
    Compile, Provided, Runtime, Test, System, Import
}

enum Resolution {
    Included, OmittedForDuplicate, OmittedForConflict
}

#[derive(PartialEq, Eq)]
struct GraphNode {
    group_id: String,
    artifact_id: String,
    dep_type: MvnDepType
}

struct MvnArtifact {
    id : GraphNode,
    numeric_id: u32,
    group_id: String,
    artifact_id: String,
    version : String,
    optional : bool,
    scopes : Vec<MvnScope>,
    types : Vec<MvnDepType>
}

struct PomDepEdge {
    from: GraphNode,
    to: GraphNode,
    numeric_from: u32,
    numeric_to: u32,
    resolution: Resolution
}

struct PomGraph {
    graph_name: String,
    artifacts: Vec<MvnArtifact>,
    dependencies: Vec<PomDepEdge>
}

impl PomGraph {

}
