use std::{collections::HashMap, fmt::Debug};

use crate::graph::{EdgeIndex, Graph, NodeIndex};

#[derive(Default)]
pub struct Hive {
    graph: Graph<(), ()>,
    node_map: HashMap<String, NodeIndex>,
}

impl Debug for Hive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.graph))
    }
}

impl Hive {
    pub fn parse<'a>(&mut self, args: &'a [&'a str]) -> &'a [&'a str] {
        match *args {
            [src, ">", dst, ref xs @ ..] | [dst, "<", src, ref xs @ ..] => {
                let src = *self.node(src);
                let dst = *self.node(dst);
                self.graph.add_edge(src, dst, ());
                xs
            }
            ["d" | "delete", ident, ref xs @ ..] => {
                if let Ok(idx) = ident.parse() {
                    if let Some(_edge) = self.graph.remove_edge(EdgeIndex(idx)) {
                        println!("Removed edge {idx}");
                        return xs;
                    }
                }

                if let Some(idx) = self.node_map.remove(ident) {
                    let _node = self.graph.remove_node_unchecked(idx);
                    println!("Removed node {ident}");
                    return xs;
                }
                args
            }
            _ => args,
        }
    }

    fn node(&mut self, node: &str) -> &NodeIndex {
        self.node_map
            .entry(node.to_string())
            .or_insert_with(|| self.graph.add_node(()))
    }
}
