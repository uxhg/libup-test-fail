use std::borrow::Cow;
use std::io::Write;
use std::vec::Vec;
use std::collections::HashSet;

use crate::dataflow::FlowGraph;

type Nd = (usize, String);
type Ed = (Nd, Nd);
pub struct DotGraph {
    edges: Vec<(usize, usize)>,
    nodes: Vec<String>,
    name: String
}

pub fn satisfy_dot_id(orig: &str) -> String {
    String::from(orig.replace(".", "_").replace("<", "__").replace(">","__"))
}

impl DotGraph {
    pub fn from_flow_graph(g: &FlowGraph) -> DotGraph {

        let nodes_vec = g.all_nodes_sorted();
        let mut edges_vec: Vec<(usize, usize)>  = vec!();
        for f in g.get_class_flows() {
            let src_id = nodes_vec.binary_search(&f.s()).unwrap();
            let dst_id = nodes_vec.binary_search(&f.d()).unwrap();
            edges_vec.push((src_id, dst_id));
        }

        DotGraph {
            name: String::from("test"),
            nodes: nodes_vec.into_iter().map(|x| String::from(x)).collect(),
            edges: edges_vec
            /*
            edges: g.get_class_flows().iter()
                .filter(|x| x.s() != "<anonymous class>" && x.d() != "<anonymous class>")
                .map(|x| (satisfy_dot_id(x.s()), satisfy_dot_id(x.d()))).collect::<Vec<Ed>>()

             */
        }
    }
}

impl<'a> dot::Labeller<'a, Nd, Ed> for DotGraph {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new(&self.name).unwrap()
    }

    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        // println!("{}", n.0);
        dot::Id::new(format!("N{}", n.0)).unwrap()
    }

    fn node_label<'b>(&'b self, n: &Nd) -> dot::LabelText<'b> {
        let &(i, _) = n;
        dot::LabelText::LabelStr(self.nodes[i][..].into())
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for DotGraph {
    /*
    fn nodes(&self) -> dot::Nodes<'a,Nd> {
        // (assumes that |N| \approxeq |E|)
        let &DotGraph {ref edges, ref name} = self;
        let mut nodes: Vec<Nd> = Vec::with_capacity(edges.len());
        for (src,dst) in edges {
            nodes.push(String::from(src));
            nodes.push(String::from(dst));
        }
        nodes.sort();
        nodes.dedup();
        Cow::Owned(nodes)
    }

    fn edges(&'a self) -> dot::Edges<'a,Ed> {
        let &DotGraph {ref edges, ref name} = self;
        Cow::Borrowed(&edges[..])
    }

     */


    fn nodes(&'a self) -> dot::Nodes<Nd> {
        self.nodes.iter().map(|s| String::from(s)).enumerate().collect()
    }

    fn edges(&'a self) -> dot::Edges<Ed> {
        self.edges.iter()
            .map(|&(i,j)|((i, self.nodes[i].clone()),
                          (j, self.nodes[j].clone())))
            .collect()
    }


    fn source(&self, e: &Ed) -> Nd { e.0.clone() }

    fn target(&self, e: &Ed) -> Nd { e.1.clone() }
}

pub fn render_to<W: Write>(g: &DotGraph, output: &mut W) {
    // let edges = DotEdges(vec!((0,1), (0,2), (1,3), (2,3), (3,4), (4,4)));
    dot::render(g, output).unwrap()
}
