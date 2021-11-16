use core::{
    mem,
    ops::{Index, IndexMut},
};

#[derive(Clone, Copy, Debug)]
struct NodeIndex(usize);

impl<N> Index<NodeIndex> for Vec<Result<Node<N>, Option<NodeIndex>>> {
    type Output = Result<Node<N>, Option<NodeIndex>>;

    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self[index.0]
    }
}

impl<N> IndexMut<NodeIndex> for Vec<Result<Node<N>, Option<NodeIndex>>> {
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self[index.0]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct EdgeIndex(usize);

impl<E> Index<EdgeIndex> for Vec<Result<Edge<E>, Option<EdgeIndex>>> {
    type Output = Result<Edge<E>, Option<EdgeIndex>>;

    fn index(&self, index: EdgeIndex) -> &Self::Output {
        &self[index.0]
    }
}

impl<E> IndexMut<EdgeIndex> for Vec<Result<Edge<E>, Option<EdgeIndex>>> {
    fn index_mut(&mut self, index: EdgeIndex) -> &mut Self::Output {
        &mut self[index.0]
    }
}

/// Links to the next edge or the associated node at the end of the list
type NogeIndex = Result<EdgeIndex, NodeIndex>;

#[derive(Debug)]
struct Node<N> {
    data: N,
    /// [dst, src]
    next: [Option<EdgeIndex>; 2],
}

#[derive(Debug)]
struct Edge<E> {
    data: E,
    /// [src, dst]
    next: [NogeIndex; 2],
}

#[derive(Debug)]
struct Graph<N, E> {
    nodes: Vec<Result<Node<N>, Option<NodeIndex>>>,
    edges: Vec<Result<Edge<E>, Option<EdgeIndex>>>,
}

impl<N, E> Index<NodeIndex> for Graph<N, E> {
    type Output = Node<N>;

    fn index(&self, index: NodeIndex) -> &Self::Output {
        self.nodes[index].as_ref().unwrap()
    }
}

impl<N, E> IndexMut<NodeIndex> for Graph<N, E> {
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        self.nodes[index].as_mut().unwrap()
    }
}

impl<N, E> Index<EdgeIndex> for Graph<N, E> {
    type Output = Edge<E>;

    fn index(&self, index: EdgeIndex) -> &Self::Output {
        self.edges[index].as_ref().unwrap()
    }
}

impl<N, E> IndexMut<EdgeIndex> for Graph<N, E> {
    fn index_mut(&mut self, index: EdgeIndex) -> &mut Self::Output {
        self.edges[index].as_mut().unwrap()
    }
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
        self.nodes.push(Ok(Node {
            data: node,
            next: [None; 2],
        }));
        node_idx
    }

    fn add_edge(&mut self, src_idx: NodeIndex, dst_idx: NodeIndex, edge: E) -> EdgeIndex {
        let edge_idx = EdgeIndex(self.edges.len());
        let src = self[src_idx].next[0].replace(edge_idx).ok_or(src_idx);
        let dst = self[dst_idx].next[1].replace(edge_idx).ok_or(dst_idx);
        self.edges.push(Ok(Edge {
            data: edge,
            next: [src, dst],
        }));
        edge_idx
    }

    fn next(&self, noge_idx: NogeIndex, dir: usize) -> NogeIndex {
        match noge_idx {
            Ok(edge_idx) => self[edge_idx].next[dir],
            Err(node_idx) => Ok(self[node_idx].next[dir].unwrap()),
        }
    }

    fn prior(&self, idx: EdgeIndex, dir: usize) -> NogeIndex {
        let mut next = self[idx].next[dir];
        loop {
            next = match (next, self.next(next, dir)) {
                (next, Ok(edge_idx)) if edge_idx == idx => return next,
                (_, next) => next,
            }
        }
    }

    fn remove_edge(&mut self, idx: EdgeIndex) -> E {
        for dir in 0..2 {
            match (self.prior(idx, dir), self[idx].next[dir]) {
                (Ok(prior_idx), Ok(next_idx)) => self[prior_idx].next[dir] = Ok(next_idx),
                (Ok(prior_idx), Err(next_idx)) => self[prior_idx].next[dir] = Err(next_idx),
                (Err(prior_idx), Ok(next_idx)) => self[prior_idx].next[dir] = Some(next_idx),
                (Err(prior_idx), Err(_next_idx)) => self[prior_idx].next[dir] = None,
            }
        }

        // TODO: Cache and recycle freed indices.
        mem::replace(&mut self.edges[idx], Err(None)).unwrap().data
    }
}

fn main() {
    println!("Hello, hive!");

    let mut graph = Graph::new();
    let n0 = graph.add_node("0");
    let n1 = graph.add_node("1");
    let n2 = graph.add_node("2");
    let e0 = graph.add_edge(n0, n2, "0");
    dbg!(&graph);
    let e1 = graph.add_edge(n1, n2, "1");
    dbg!(&graph);
    graph.remove_edge(e0);
    dbg!(&graph);
    graph.remove_edge(e1);
    dbg!(&graph);
}
