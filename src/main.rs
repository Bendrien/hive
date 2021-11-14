#[derive(Clone, Copy, Debug)]
struct NodeIndex(usize);

#[derive(Clone, Copy, Debug, PartialEq)]
struct EdgeIndex(usize);

#[derive(Debug)]
struct Node<N> {
    data: N,
    link: Option<EdgeIndex>,
}

#[derive(Debug)]
struct Edge<E> {
    data: E,
    /// Links to the next edge or the destination node at the end
    link: Result<EdgeIndex, NodeIndex>,
    node: NodeIndex,
}

#[derive(Debug)]
struct Graph<N, E> {
    nodes: Vec<Node<N>>,
    edges: Vec<Edge<E>>,
}

impl<N, E> Graph<N, E> {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    fn add_node(&mut self, node: N) -> NodeIndex {
        let node_idx = NodeIndex(self.nodes.len());
        self.nodes.push(Node {
            data: node,
            link: None,
        });
        node_idx
    }

    fn add_edge(&mut self, src_idx: NodeIndex, dst_idx: NodeIndex, edge: E) -> EdgeIndex {
        let edge_idx = EdgeIndex(self.edges.len());
        let dst_node = self.nodes.get_mut(dst_idx.0).unwrap();
        self.edges.push(Edge {
            data: edge,
            link: dst_node.link.replace(edge_idx).ok_or(dst_idx),
            node: src_idx,
        });
        edge_idx
    }

    fn predecessor(&self, idx: EdgeIndex) -> Result<EdgeIndex, NodeIndex> {
        let mut tmp = idx;
        loop {
            let link = self.edges[tmp.0].link;
            let edge_idx = link.unwrap_or_else(|idx| self.nodes[idx.0].link.unwrap());
            if edge_idx == idx {
                return link;
            }
            tmp = edge_idx;
        }
    }

    fn remove_edge(&mut self, idx: EdgeIndex) -> Option<E> {
        match self.edges[idx.0].link {
            Ok(successor_idx) => match self.predecessor(idx) {
                Ok(edge_idx) => self.edges[edge_idx.0].link = Ok(successor_idx),
                Err(node_idx) => self.nodes[node_idx.0].link = Some(successor_idx),
            },
            Err(node_idx) => match self.predecessor(idx) {
                Ok(edge_idx) => self.edges[edge_idx.0].link = Err(node_idx),
                Err(node_idx) => self.nodes[node_idx.0].link = None,
            },
        }
        // TODO: Preserve indices!
        Some(self.edges.remove(idx.0).data)
    }
}

fn main() {
    println!("Hello, hive!");

    let mut graph = Graph::new();
    let n1 = graph.add_node(());
    let n2 = graph.add_node(());
    let n3 = graph.add_node(());
    let e1 = graph.add_edge(n1, n3, ());
    let _e2 = graph.add_edge(n2, n3, ());
    dbg!(&graph);
    graph.remove_edge(e1);
    dbg!(&graph);
}
