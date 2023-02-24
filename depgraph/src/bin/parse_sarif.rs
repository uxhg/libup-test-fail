use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use depgraph::sarif::sarif;
use depgraph::sarif::sarif::Sarif;
use depgraph::utils::utils;

fn main() {
    utils::init_log();
    let p = Path::new("/tmp/queries_-1357352-QZFVjznp456X/interpretedResults52.sarif");
    match sarif::read_from_json(p) {
        Some(s) => {
            println!("{}", s.schema())
        }
        None => {
            return;
        }
    }

    let path_vec = vec![
        "/home/wuxh/Projects/lib-conflict/libup-test-fail/depgraph/data/SARIF/fastjson/param.sarif",
        "/home/wuxh/Projects/lib-conflict/libup-test-fail/depgraph/data/SARIF/fastjson/receiver.sarif"];
    let sarif_vec: Vec<Sarif> = path_vec.iter().filter_map(|x| sarif::read_from_json(x)).collect();
    let merged_flows = sarif::merge_paths(sarif_vec);
    let paths_writer = BufWriter::new(File::create("call-ctx.json")
        .expect("Cannot open or create call-ctx file"));
    let stringify_merged_flows: HashMap<String, HashMap<String, Vec<String>>> = merged_flows.into_iter()
        .map(|(x, y)|
            (x, y.into_iter().map(|(a, b)| (a.to_string(), b))
                .collect::<HashMap<String, Vec<String>>>()))
        .collect();
    serde_json::ser::to_writer_pretty(paths_writer, &stringify_merged_flows);
}