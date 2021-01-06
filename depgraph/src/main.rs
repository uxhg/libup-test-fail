mod dataflow;
mod pomdep;


fn main() {
	let mut x = dataflow::FlowGraph:: from_csv(String::from("data/openscoring-result.csv")).unwrap();
	for f in x.get_lib_flows() {
		println!("{:?}", f);
	}
}

