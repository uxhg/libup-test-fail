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
    sarif::merge_paths(sarif_vec);
}