use std::io::Write;
use std::vec::Vec;

use crate::dataflow::FlowGraph;

type Nd = (usize, String);
type Ed = (Nd, Nd);

pub struct DotGraph {
    edges: Vec<(usize, usize)>,
    nodes: Vec<String>,
    name: String,
}

pub fn satisfy_dot_id(orig: &str) -> String {
    String::from(orig.replace(".", "_").replace("<", "__").replace(">", "__"))
}

impl DotGraph {
    pub fn edges(&self) -> &Vec<(usize, usize)> {
        &self.edges
    }
    pub fn nodes(&self) -> &Vec<String> {
        &self.nodes
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn from_flow_graph(g: &FlowGraph) -> DotGraph {
        let nodes_vec = g.all_nodes_sorted();
        let mut edges_vec: Vec<(usize, usize)> = vec!();
        for f in g.get_class_flows() {
            let src_id = nodes_vec.binary_search(&f.s()).unwrap();
            let dst_id = nodes_vec.binary_search(&f.d()).unwrap();
            edges_vec.push((src_id, dst_id));
        }

        DotGraph {
            name: String::from("test"),
            nodes: nodes_vec.into_iter().map(|x| String::from(x)).collect(),
            edges: edges_vec,
            /*
            edges: g.get_class_flows().iter()
                .filter(|x| x.s() != "<anonymous class>" && x.d() != "<anonymous class>")
                .map(|x| (satisfy_dot_id(x.s()), satisfy_dot_id(x.d()))).collect::<Vec<Ed>>()

             */
        }
    }

    pub fn render_to<W: Write>(&self, output: &mut W) {
        // let edges = DotEdges(vec!((0,1), (0,2), (1,3), (2,3), (3,4), (4,4)));
        dot::render(self, output).unwrap()
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

    fn node_label(&self, n: &Nd) -> dot::LabelText {
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
            .map(|&(i, j)| ((i, self.nodes[i].clone()),
                            (j, self.nodes[j].clone())))
            .collect()
    }


    fn source(&self, e: &Ed) -> Nd { e.0.clone() }

    fn target(&self, e: &Ed) -> Nd { e.1.clone() }
}

pub struct DotStyle {
    shape: Option<String>,
    style: Option<String>,
    fontname: Option<String>,
    fontsize: Option<String>,
    indent: usize,
}

impl Default for DotStyle {
    fn default()  -> Self {
        DotStyle {
            shape: Some(String::from("box")),
            style: Some(String::from("rounded")),
            fontname: Some(String::from("Helvetica")),
            fontsize: Some(String::from("14")),
            indent: 2
        }
    }
}
impl DotStyle {
    pub fn new(shape: Option<String>, style: Option<String>, fontname: Option<String>, fontsize: Option<String>) -> Self {
        DotStyle { shape, style, fontname, fontsize, indent: 2}
    }

    pub fn create_with_all(shape: &str, style: &str, fontname: &str, fontsize: &str, indent: usize) -> Self {
        DotStyle {
            shape: Some(String::from(shape)),
            style: Some(String::from(style)),
            fontname: Some(String::from(fontname)),
            fontsize: Some(String::from(fontsize)),
            indent
        }
    }
    pub fn shape(&self) -> &Option<String> {
        &self.shape
    }
    pub fn style(&self) -> &Option<String> {
        &self.style
    }
    pub fn fontname(&self) -> &Option<String> {
        &self.fontname
    }
    pub fn fontsize(&self) -> &Option<String> {
        &self.fontsize
    }
    pub fn indent(&self) -> usize {
        self.indent
    }
    pub fn create_style_sheet(&self) -> Vec<String> {
        let mut style_sheet: Vec<String> = Vec::new();
        if self.shape().is_some() {
            style_sheet.push(format!("shape=\"{}\"", self.shape().as_ref().unwrap()));
        }
        if self.style().is_some() {
            style_sheet.push(format!("style=\"{}\"", self.style().as_ref().unwrap()));
        }
        if self.fontname().is_some() {
            style_sheet.push(format!("fontname=\"{}\"", self.fontname().as_ref().unwrap()));
        }
        if self.fontsize().is_some() {
            style_sheet.push(format!("fontsize=\"{}\"", self.fontsize().as_ref().unwrap()));
        }
        style_sheet
    }

    pub fn node_style_decl(&self) -> String {
        let ss = self.create_style_sheet();
        format!("node [{}]", ss.join(","))
    }

    pub fn edge_style_decl(&self) -> String {
        let ss: Vec<String> = self.create_style_sheet().into_iter().filter(|x| x.starts_with("fontname") || x.starts_with("fontsize")).collect();
        format!("edge [{}]", ss.join(","))
    }
}

#[cfg(test)]
mod test{
    use crate::dot_graph::DotStyle;

    #[test]
    fn test_node_style_decl(){
        let ss = DotStyle::create_with_all("box", "rounded", "Avenir", "14", 2);
        assert_eq!(ss.node_style_decl(), "node [shape=\"box\",style=\"rounded\",fontname=\"Avenir\",fontsize=\"14\"]");
    }

    #[test]
    fn test_edge_style_decl(){
        let ss = DotStyle::create_with_all("box", "rounded", "Avenir", "14", 2);
        assert_eq!(ss.edge_style_decl(), "edge [fontname=\"Avenir\",fontsize=\"14\"]");
    }
}

