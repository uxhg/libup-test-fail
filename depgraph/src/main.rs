// mod jar_class_map;
mod dataflow;
mod pomdep;
mod jar_class_map;


fn main() {
	let mut x = dataflow::FlowGraph:: from_csv(String::from("data/docker-java-result.csv")).unwrap();
	for f in x.get_lib_flows() {
		println!("{:?}", f);
	}

	let y = pomdep::PomGraph::read_from_json("data/docker-java-pom.json");
    println!("{}", y.unwrap().graph_name());
}

