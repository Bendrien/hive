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
    fn track(&mut self, action: Rc<dyn Fn(&mut Hive)>) {
        if self.redo {
            // While "redoing" we ignore all the implicitly incoming undo of the redo actions!
            // Question: Provide action as a generic/impl argument to avoid heap allocation?
            return;
        }
        if self.pos == self.history.len() {
            self.pos = self.history.len() + 1
        }
        self.history.push(action);
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

impl Debug for Hive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}/{}|{:?}",
            self.undo.pos,
            self.undo.history.len(),
            self.graph
        ))
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
                        println!("Removed edge {idx}");
                        return xs;
                    }
                }

                if self.remove_node(ident) {
                    self.undo.reset();
                    println!("Removed node {ident}");
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
            self.undo.track(Rc::new({
                let node = node.to_string();
                move |hive: &mut Hive| {
                    assert!(hive.remove_node(&node));
                }
            }));
            self.graph.add_node(())
        })
    }

    fn remove_node(&mut self, node: &str) -> bool {
        if let Some(idx) = self.nodes.remove(node) {
            let _node = self.graph.remove_node_unchecked(idx);
            self.undo.track(Rc::new({
                let node = node.to_string();
                move |hive| {
                    hive.add_node(&node);
                }
            }));
            return true;
        }
        false
    }

    fn add_edge(&mut self, src: NodeIndex, dst: NodeIndex) {
        let edge = self.graph.add_edge(src, dst, ());
        self.undo.track(Rc::new(move |hive| {
            // FIXME: No assert for now because an explicitly removed node may had implicitly removed this edge
            hive.remove_edge(edge);
        }));
    }

    fn remove_edge(&mut self, edge: EdgeIndex) -> bool {
        if let Some([src, dst]) = self.graph.src_dst(edge) {
            let _edge = self.graph.remove_edge_unchecked(edge);
            self.undo
                .track(Rc::new(move |hive| hive.add_edge(src, dst)));
            return true;
        }
        false
    }
}
