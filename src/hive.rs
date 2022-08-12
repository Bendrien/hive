use std::{collections::HashMap, fmt::Debug, rc::Rc};

use crate::graph::{EdgeIndex, Graph, NodeIndex};

#[derive(Default)]
pub struct Hive {
    graph: Graph<(), ()>,
    nodes: HashMap<String, NodeIndex>,
    pub undo: Undo,
}

#[derive(Default)]
pub struct Undo {
    history: Vec<Rc<dyn Fn(&mut Hive)>>,
    pos: usize,
    redo: bool,
}

impl Undo {
    fn track<F>(&mut self, action: F)
    where
        F: Fn(&mut Hive) + 'static,
    {
        if self.redo {
            // While "redoing" we ignore all the implicitly incoming undo of the redo actions!
            // FIXME: On redo we get an action just to drop it right away. Can we avoid its construction in the first place?
            return;
        }
        if self.pos == self.history.len() {
            self.pos = self.history.len() + 1
        }
        self.history.push(Rc::new(action));
    }

    fn reset(&mut self) {
        self.pos = self.history.len();
    }

    pub fn snapshot(&self) -> usize {
        self.history.len()
    }

    pub fn pile(&mut self, snapshot: usize) {
        if self.history.len() < 2 || self.snapshot().saturating_sub(snapshot) < 2 {
            // Building a pile with less then 2 elements equals doing nothing here.
            return;
        }
        assert!(!self.redo);

        let pile = self.history.split_off(snapshot);
        self.history.push(Rc::new(move |hive| {
            let snapshot = hive.undo.snapshot();
            for action in pile.iter().rev() {
                action(hive)
            }
            hive.undo.pile(snapshot);
        }));
        if self.pos > self.history.len() {
            self.pos = self.history.len()
        }
    }
}

impl Hive {
    pub fn parse<'a>(&mut self, args: &'a [&'a str]) -> &'a [&'a str] {
        match *args {
            [src, ">", dst, ref xs @ ..] | [dst, "<", src, ref xs @ ..] => {
                let snapshot = self.undo.snapshot();
                let src = *self.add_node(src);
                let dst = *self.add_node(dst);
                self.add_edge(src, dst);
                self.undo.pile(snapshot);
                self.undo.reset();
                xs
            }
            ["d" | "delete", ident, ref xs @ ..] => {
                if let Ok(idx) = ident.parse() {
                    if self.remove_edge(EdgeIndex(idx)) {
                        self.undo.reset();
                        return xs;
                    }
                }

                let snapshot = self.undo.snapshot();
                if self.remove_node(ident) {
                    self.undo.pile(snapshot);
                    self.undo.reset();
                    return xs;
                }
                args
            }
            ["u" | "undo", ref xs @ ..] => {
                if let Some(pos) = self.undo.pos.checked_sub(1) {
                    self.undo.history[pos].clone()(self);
                    self.undo.pos = pos;
                } else {
                    println!("Nothing to undo")
                }
                xs
            }
            ["r" | "redo", ref xs @ ..] => {
                let pos = self.undo.pos + 1;
                if pos < self.undo.history.len() {
                    self.undo.redo = true;
                    // The most recent undo of an undo aka redo is being tracked at the end of the history!
                    // Its origin is still available at idx and has the same effect. Therefore we can remove
                    // the redo from the history here.
                    self.undo.history.pop().unwrap()(self);
                    self.undo.redo = false;
                    self.undo.pos = pos;
                } else {
                    println!("Nothing to redo")
                }
                xs
            }
            ["p" | "pile", n, ref xs @ ..] => {
                if let Ok(n) = n.parse() {
                    let snapshot = self.undo.snapshot();
                    self.undo.pile(snapshot.saturating_sub(n));
                    return xs;
                }
                args
            }
            _ => args,
        }
    }

    fn add_node(&mut self, node: &str) -> &NodeIndex {
        self.nodes.entry(node.to_string()).or_insert_with(|| {
            self.undo.track({
                let node = node.to_string();
                move |hive: &mut Hive| assert!(hive.remove_node(&node))
            });
            self.graph.add_node(())
        })
    }

    fn remove_node(&mut self, node: &str) -> bool {
        if let Some(idx) = self.nodes.remove(node) {
            let mut edges = (0..2)
                .flat_map(|dir| self.graph.edges(idx, dir))
                .collect::<Vec<_>>();
            edges.sort();
            for edge in edges.into_iter().rev() {
                self.remove_edge(edge);
            }
            let _node = self.graph.remove_node_unchecked(idx);
            self.undo.track({
                let node = node.to_string();
                move |hive| {
                    hive.add_node(&node);
                }
            });
            return true;
        }
        false
    }

    fn add_edge(&mut self, src: NodeIndex, dst: NodeIndex) {
        let edge = self.graph.add_edge(src, dst, ());
        self.undo.track(move |hive| assert!(hive.remove_edge(edge)));
    }

    fn remove_edge(&mut self, edge: EdgeIndex) -> bool {
        if let Some([src, dst]) = self.graph.src_dst(edge) {
            let _edge = self.graph.remove_edge_unchecked(edge);
            self.undo.track(move |hive| hive.add_edge(src, dst));
            return true;
        }
        false
    }
}

impl Debug for Hive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const DIR_STR: [&str; 2] = [">", "<"];
        f.write_fmt(format_args!(
            "{}/{}|{}/{}|{}/{}\n",
            self.undo.pos,
            self.undo.history.len(),
            self.graph.nodes.iter().filter(|n| n.is_ok()).count(),
            self.graph.nodes.len(),
            self.graph.edges.iter().filter(|n| n.is_ok()).count(),
            self.graph.edges.len(),
        ))?;
        for dir in 0..1 {
            for idx in 0..self.graph.nodes.len() {
                if self.graph.nodes[idx].is_err() {
                    continue;
                }
                let neighbors: Vec<_> = self.graph.neighbors(NodeIndex(idx), dir).collect();
                if neighbors.is_empty() {
                    continue;
                }
                let idx = self
                    .nodes
                    .iter()
                    .find(|(_, node)| node.0 == idx)
                    .map(|(ident, _)| ident)
                    .unwrap();
                f.write_fmt(format_args!("{idx} {}", DIR_STR[dir]))?;
                let last_idx = neighbors.len() - 1;
                for (idx, (NodeIndex(node), EdgeIndex(edge))) in neighbors.into_iter().enumerate() {
                    let node = self
                        .nodes
                        .iter()
                        .find(|(_, idx)| node == idx.0)
                        .map(|(ident, _)| ident)
                        .unwrap();
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
