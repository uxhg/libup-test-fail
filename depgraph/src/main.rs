// mod jar_class_map;
use env_logger::Env;

mod dataflow;
mod pomdep;
mod jar_class_map;

fn init_log() {
    let env = Env::default()
        .filter_or("RUST_LOG", "warn")
        .write_style_or("LOG_STYLE", "always");
    env_logger::init_from_env(env);
}

fn main() {
    init_log();
    // let mut x = dataflow::FlowGraph:: from_csv(String::from("data/docker-java-result.csv")).unwrap();
    // for f in x.get_lib_flows() {
    // 	println!("{:?}", f);
    // }

    let y = pomdep::PomGraph::read_from_json("data/docker-java-pom.json");
    println!("{}", y.unwrap().graph_name());

    let z = jar_class_map::MvnModule::new(
        "docker-java",
        "/home/wuxh/Projects/lib-conflict/cases/docker-java");
        //"/home/wuxh/Projects/lib-conflict/cases/openscoring-codeql/openscoring-client");
    println!("{}", z.name());
    for j in z.jar_map() {
        println!("{}", j.0);
        for j_art in j.1.artifacts() {
            println!("{}", j_art.0)
        }
    }
}

