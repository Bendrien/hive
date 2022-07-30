use core::{
    mem,
    ops::{Index, IndexMut},
};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

impl<N, E> std::fmt::Debug for Graph<N, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const DIR_STR: [&str; 2] = [">", "<"];
        f.write_fmt(format_args!(
            "{}/{}|{}/{}\n",
            self.nodes.iter().filter(|n| n.is_ok()).count(),
            self.nodes.len(),
            self.edges.iter().filter(|n| n.is_ok()).count(),
            self.edges.len(),
        ))?;
        for dir in 0..2 {
            for idx in 0..self.nodes.len() {
                if let Err(_) = self.nodes[idx] {
                    continue;
                }
                let neighbors: Vec<_> = self.neighbors(NodeIndex(idx), dir).collect();
                if neighbors.is_empty() {
                    continue;
                }
                f.write_fmt(format_args!("{idx} {}", DIR_STR[dir]))?;
                let last_idx = neighbors.len() - 1;
                for (idx, (NodeIndex(node), EdgeIndex(edge))) in neighbors.into_iter().enumerate() {
                    f.write_fmt(format_args!(
                        " {node}|{edge}{}",
                        if idx == last_idx { "\n" } else { "," }
                    ))?;
                }
            }
        }
        Ok(())
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

    fn next(&self, idx: NogeIndex, dir: usize) -> NogeIndex {
        match idx {
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

    /// Gather traversal information with respect to the given node and direction.
    pub fn schedule(&self, idx: NodeIndex, dir: usize) -> HashMap<NodeIndex, ScheduleInfo> {
        let mut queue = VecDeque::new();
        let mut schedule = HashMap::new();
        queue.push_front((idx, 0));
        schedule.insert(idx, ScheduleInfo::new(0, self.edges(idx, dir).count(), dir));

        while let Some((idx, stage)) = queue.pop_front() {
            if self[idx].next[dir].is_some() {
                let next_stage = stage + 1;
                for (neighbor, _) in self.neighbors(idx, dir) {
                    schedule
                        .entry(neighbor)
                        .and_modify(|info| info.update(next_stage, dir))
                        .or_insert_with(|| {
                            queue.push_back((neighbor, next_stage));
                            ScheduleInfo::new(next_stage, self.edges(neighbor, dir).count(), dir)
                        });
                }
            }
        }
        return schedule;
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
            iter: self.edges(idx, dir),
        }
    }

    pub fn bfs(&self, idx: NodeIndex, dir: usize) -> Bfs<'_, N, E> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        queue.push_front(idx);
        visited.insert(idx);

        Bfs {
            graph: &self,
            queue,
            visited,
            dir,
        }
    }
}

#[derive(Debug)]
pub struct ScheduleInfo {
    stage: usize,
    count: [usize; 2],
}

impl ScheduleInfo {
    fn new(stage: usize, count: usize, dir: usize) -> Self {
        let mut info = ScheduleInfo {
            count: [0; 2],
            stage,
        };
        info.count[dir] = count;
        info
    }

    fn update(&mut self, stage: usize, dir: usize) {
        self.stage = stage;
        self.count[dir ^ 1] += 1;
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
    iter: Edges<'a, E>,
}

impl<E> Iterator for Neighbors<'_, E> {
    type Item = (NodeIndex, EdgeIndex);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(edge_idx) = self.iter.next() {
            // Follow the other edge list until reaching its end, holding the node index
            let node_dir = self.iter.dir ^ 1;
            let mut next = self.iter.edges[edge_idx.0].as_ref().unwrap().next[node_dir];
            while let Ok(next_idx) = next {
                next = self.iter.edges[next_idx.0].as_ref().unwrap().next[node_dir];
            }

            return next.err().map(|node_idx| (node_idx, edge_idx));
        }
        None
    }
}

pub struct Bfs<'a, N, E> {
    graph: &'a Graph<N, E>,
    queue: VecDeque<NodeIndex>,
    visited: HashSet<NodeIndex>,
    dir: usize,
}

impl<N, E> Iterator for Bfs<'_, N, E> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.queue.pop_front() {
            for (neighbor, _) in self.graph.neighbors(idx, self.dir) {
                if self.visited.insert(neighbor) {
                    self.queue.push_back(neighbor);
                }
            }

            return Some(idx);
        }
        None
    }
}
