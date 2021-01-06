use serde::Deserialize;

mod dataflow {
    #[derive(Deserialize)]
    struct Flow {
        src_method: String,
        src_artifact: String,
        dst_method: String,
        dst_artifact: String
    }

    struct FlowGraph {
        flows: Vec<Flow>
    }

    impl FlowGraph{
        fn from_csv(file: ) {
            let mut reader = csv::Reader::from_reader(csv.as_bytes());
        }
    }
}
