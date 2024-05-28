use dot::{Edges, Nodes};
use rustc_hir::def_id::DefId;
use rustc_hir::HirId;
use std::borrow::Cow;
use std::cmp::PartialEq;

#[derive(Debug, Clone)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone)]
pub struct Node {
    id: usize,
    label: String,
    pub kind: NodeKind,
    pub panics: bool,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    LocalFn(HirId),
    NonLocalFn(DefId),
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub call_id: HirId,
    pub ty: Option<String>,
}

impl<'a> dot::Labeller<'a, Node, Edge> for Graph {
    fn graph_id(&self) -> dot::Id<'a> {
        dot::Id::new("call_graph").unwrap()
    }

    fn node_id(&self, n: &Node) -> dot::Id<'a> {
        dot::Id::new(format!("node{:?}", n.id)).unwrap()
    }

    fn node_label(&self, n: &Node) -> dot::LabelText<'a> {
        dot::LabelText::label(n.label.clone())
    }

    fn edge_label(&self, e: &Edge) -> dot::LabelText<'a> {
        dot::LabelText::label(e.ty.clone().unwrap_or(String::from("unknown")))
    }
}

impl<'a> dot::GraphWalk<'a, Node, Edge> for Graph {
    fn nodes(&'a self) -> Nodes<'a, Node> {
        let mut nodes = vec![];
        for edge in &self.edges {
            if !nodes.contains(&self.nodes[edge.from]) {
                nodes.push(self.nodes[edge.from].clone());
            }
            if !nodes.contains(&self.nodes[edge.to]) {
                nodes.push(self.nodes[edge.to].clone());
            }
        }
        Cow::Owned(nodes)
    }

    fn edges(&'a self) -> Edges<'a, Edge> {
        Cow::Owned(self.edges.clone())
    }

    fn source(&'a self, edge: &Edge) -> Node {
        self.get_node(edge.from)
            .expect("Node at edge's start does not exist!")
            .clone()
    }

    fn target(&'a self, edge: &Edge) -> Node {
        self.get_node(edge.to)
            .expect("Node at edge's end does not exist!")
            .clone()
    }
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, label: &str, node_kind: NodeKind) -> usize {
        let node = Node::new(self.nodes.len(), label, node_kind);
        let id = node.id();
        self.nodes.push(node);
        id
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    pub fn get_node(&self, id: usize) -> Option<Node> {
        return if id < self.nodes.len() {
            Some(self.nodes[id].clone())
        } else {
            None
        };
    }

    pub fn find_local_fn_node(&self, id: HirId) -> Option<Node> {
        for node in &self.nodes {
            if let NodeKind::LocalFn(hir_id) = node.kind {
                if hir_id == id {
                    return Some(node.clone());
                }
            }
        }

        None
    }

    pub fn find_non_local_fn_node(&self, id: DefId) -> Option<Node> {
        for node in &self.nodes {
            if let NodeKind::NonLocalFn(def_id) = node.kind {
                if def_id == id {
                    return Some(node.clone());
                }
            }
        }

        None
    }

    pub fn to_dot(&self) -> String {
        let mut buf = Vec::new();

        dot::render(self, &mut buf).unwrap();

        String::from_utf8(buf).unwrap()
    }

    pub fn incoming_edges(&mut self, node: &Node) -> Vec<&mut Edge> {
        let mut res = vec![];

        for edge in &mut self.edges {
            if edge.to == node.id {
                res.push(edge);
            }
        }

        res
    }
}

impl Node {
    fn new(node_id: usize, label: &str, node_type: NodeKind) -> Self {
        Node {
            id: node_id,
            label: String::from(label),
            kind: node_type,
            panics: false,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

impl NodeKind {
    pub fn local_fn(id: HirId) -> Self {
        NodeKind::LocalFn(id)
    }

    pub fn non_local_fn(id: DefId) -> Self {
        NodeKind::NonLocalFn(id)
    }

    pub fn def_id(&self) -> DefId {
        match self {
            NodeKind::LocalFn(hir_id) => hir_id.owner.to_def_id(),
            NodeKind::NonLocalFn(def_id) => def_id.clone(),
        }
    }
}

impl Edge {
    pub fn new(from: usize, to: usize, call_id: HirId) -> Self {
        Edge {
            from,
            to,
            call_id,
            ty: None,
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.kind == other.kind
    }
}

impl PartialEq for NodeKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NodeKind::LocalFn(id1), NodeKind::LocalFn(id2)) => id1 == id2,
            (NodeKind::NonLocalFn(id1), NodeKind::NonLocalFn(id2)) => id1 == id2,
            _ => false,
        }
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.to == other.to && self.from == other.from
    }
}
