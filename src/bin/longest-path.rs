
use std::io::prelude::*;

use longest_path_rs::Graph;


fn main() {
    let mut buf: Vec<u8> = Vec::new();
    if let Err(e) = std::io::stdin().read_to_end(&mut buf) {
        eprintln!("IO Error: {}", e);
        return;
    }
    let json = match String::from_utf8(buf) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("UTF-8 Error: {}", e);
            return;
        }
    };
    let graph = match Graph::from_json_undirected(&json) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("JSON Error: {}", e);
            return;
        }
    };
    println!("Graph: {:?}", graph);
    let root_label = match std::env::args().nth(1) {
        Some(s) => s,
        None => {
            eprintln!("Usage: {} <root node label> [<target node label>]", std::env::args().nth(0).unwrap());
            return;
        }
    };
    let root = match graph.find_node_id(&root_label) {
        Some(id) => id,
        None => {
            eprintln!("Node not found: {}", root_label);
            return;
        }
    };
    println!("Root: {}", root_label);

    let target: Option<u32> = match std::env::args().nth(2) {
        Some(s) => match graph.find_node_id(&s) {
            Some(id) => Some(id),
            None => {
                eprintln!("Node not found: {}", s);
                return;
            }
        },
        None => None,
    };

    if let Some(target) = target {
        println!("Longest path to {}:", graph.nodes[target as usize].label);
        let longest_path = graph.find_longest_path_to(root, target);
        println!("");
        for node_id in &longest_path.node_ids {
            let node = &graph.nodes[*node_id as usize];
            println!("{}", node.label);
        }
        println!("Distance sum: {}", longest_path.distance_sum);
        return;
    }
    println!("Longest path:");
    let longest_path = graph.find_longest_path(root);
    for node_id in &longest_path.node_ids {
        let node = &graph.nodes[*node_id as usize];
        println!("{}", node.label);
    }
    println!("Distance sum: {}", longest_path.distance_sum);
}
