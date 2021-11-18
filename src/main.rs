mod graph;

use crate::graph::Graph;

fn main() {
    println!("Hello, hive!");

    let mut graph = Graph::new();
    let n0 = graph.add_node("0");
    let n1 = graph.add_node("1");
    let n2 = graph.add_node("2");
    let n3 = graph.add_node("3");
    let n4 = graph.add_node("4");

    let _e0 = graph.add_edge(n0, n2, "02");
    let _e1 = graph.add_edge(n1, n2, "12");
    let e04 = graph.add_edge(n0, n4, "04");
    let _e3 = graph.add_edge(n2, n3, "23");
    let _e4 = graph.add_edge(n2, n4, "24");

    dbg!(&graph);
    graph.remove_edge(e04);
    dbg!(&graph);
    graph.remove_node(n2);
    dbg!(&graph);
}
