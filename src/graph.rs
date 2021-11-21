use core::{
    mem,
    ops::{Index, IndexMut},
};

#[derive(Clone, Copy, Debug)]
pub struct NodeIndex(usize);

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
pub struct EdgeIndex(usize);

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
pub struct Node<N> {
    /// [dst, src]
    next: [Option<EdgeIndex>; 2],
    data: N,
}

#[derive(Debug)]
pub struct Edge<E> {
    /// [src, dst]
    next: [NogeIndex; 2],
    data: E,
}

#[derive(Debug)]
pub struct Graph<N, E> {
    // TODO: Cache and recycle freed indices in Err(Some(cache)).
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
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: N) -> NodeIndex {
        let node_idx = NodeIndex(self.nodes.len());
        self.nodes.push(Ok(Node {
            data: node,
            next: [None; 2],
        }));
        node_idx
    }

    pub fn add_edge(&mut self, src_idx: NodeIndex, dst_idx: NodeIndex, edge: E) -> EdgeIndex {
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

    fn unchain(&mut self, idx: EdgeIndex, dir: usize) {
        match (self.prior(idx, dir), self[idx].next[dir]) {
            (Ok(src_idx), dst_idx) => self[src_idx].next[dir] = dst_idx,
            (Err(src_idx), dst_idx) => self[src_idx].next[dir] = dst_idx.ok(),
        }
    }

    pub fn remove_edge(&mut self, idx: EdgeIndex) -> E {
        for dir in 0..2 {
            self.unchain(idx, dir);
        }
        mem::replace(&mut self.edges[idx], Err(None)).unwrap().data
    }

    pub fn remove_node(&mut self, idx: NodeIndex) -> N {
        for dir in 0..2 {
            while let Some(edge_idx) = self[idx].next[dir] {
                self[idx].next[dir] = self[edge_idx].next[dir].ok();
                self.unchain(edge_idx, dir ^ 1);
                self.edges[edge_idx] = Err(None);
            }
        }
        mem::replace(&mut self.nodes[idx], Err(None)).unwrap().data
    }

    pub fn edges(&self, idx: NodeIndex, dir: usize) -> Edges<'_, E> {
        Edges {
            edges: &self.edges,
            next: self[idx].next[dir],
            dir,
        }
    }

    pub fn neighbors(&self, idx: NodeIndex, dir: usize) -> Neighbors<'_, E> {
        Neighbors {
            edges: &self.edges,
            next: self[idx].next[dir],
            dir,
        }
    }
}

pub struct Edges<'a, E> {
    edges: &'a [Result<Edge<E>, Option<EdgeIndex>>],
    next: Option<EdgeIndex>,
    dir: usize,
}

impl<E> Iterator for Edges<'_, E> {
    type Item = EdgeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.next {
            self.next = self.edges[idx.0].as_ref().unwrap().next[self.dir].ok();
            return Some(idx);
        }
        None
    }
}

pub struct Neighbors<'a, E> {
    edges: &'a [Result<Edge<E>, Option<EdgeIndex>>],
    next: Option<EdgeIndex>,
    dir: usize,
}

impl<E> Iterator for Neighbors<'_, E> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.next {
            self.next = self.edges[idx.0].as_ref().unwrap().next[self.dir].ok();

            // Follow the other edge list until reaching its end, holding the node index
            let node_dir = self.dir ^ 1;
            let mut next = self.edges[idx.0].as_ref().unwrap().next[node_dir];
            while let Ok(next_idx) = next {
                next = self.edges[next_idx.0].as_ref().unwrap().next[node_dir];
            }
            return next.err();
        }
        None
    }
}
