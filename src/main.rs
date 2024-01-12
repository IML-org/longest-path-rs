
#![allow(dead_code)]

use std::io::prelude::*;
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EdgeData (String, String, u64);

#[derive(Debug, Clone, Copy)]
struct Edge {
    to: u32,
    cost: u64,
}

#[derive(Debug, Clone)]
struct Node {
    id: u32,
    edges: Vec<Edge>,
    label: String,
}

#[derive(Debug, Clone)]
struct Graph {
    node_count: u32,
    nodes: Vec<Node>,
}

impl Graph {
    fn from_json_directed(json: &str) -> Result<Graph, serde_json::Error> {
        let data: Vec<EdgeData> = serde_json::from_str(json)?;
        let mut nodes: Vec<Node> = Vec::new();
        let mut node_count: u32 = 0;
        for edge in data {
            let mut from_node_id: Option<u32> = None;
            let mut to_node_id: Option<u32> = None;
            for node in &mut nodes {
                if node.label == edge.0 {
                    from_node_id = Some(node.id);
                }
                if node.label == edge.1 {
                    to_node_id = Some(node.id);
                }
            }
            if from_node_id.is_none() {
                nodes.push(Node {
                    id: node_count,
                    edges: Vec::new(),
                    label: edge.0.clone(),
                });
                from_node_id = Some(node_count);
                node_count += 1;
            }
            if to_node_id.is_none() {
                nodes.push(Node {
                    id: node_count,
                    edges: Vec::new(),
                    label: edge.1.clone(),
                });
                to_node_id = Some(node_count);
                node_count += 1;
            }
            let from_node_id = from_node_id.unwrap();
            let to_node_id = to_node_id.unwrap();
            let from_node = &mut nodes[from_node_id as usize];
            from_node.edges.push(Edge {
                to: to_node_id,
                cost: edge.2,
            });
        }
        Ok(Graph {
            node_count,
            nodes,
        })
    }

    fn from_json_undirected(json: &str) -> Result<Graph, serde_json::Error> {
        let data: Vec<EdgeData> = serde_json::from_str(json)?;
        let mut nodes: Vec<Node> = Vec::new();
        let mut node_count: u32 = 0;
        for edge in data {
            let mut from_node_id: Option<u32> = None;
            let mut to_node_id: Option<u32> = None;
            for node in &mut nodes {
                if node.label == edge.0 {
                    from_node_id = Some(node.id);
                }
                if node.label == edge.1 {
                    to_node_id = Some(node.id);
                }
            }
            if from_node_id.is_none() {
                nodes.push(Node {
                    id: node_count,
                    edges: Vec::new(),
                    label: edge.0.clone(),
                });
                from_node_id = Some(node_count);
                node_count += 1;
            }
            if to_node_id.is_none() {
                nodes.push(Node {
                    id: node_count,
                    edges: Vec::new(),
                    label: edge.1.clone(),
                });
                to_node_id = Some(node_count);
                node_count += 1;
            }
            let from_node_id = from_node_id.unwrap();
            let to_node_id = to_node_id.unwrap();
            let from_node = &mut nodes[from_node_id as usize];
            from_node.edges.push(Edge {
                to: to_node_id,
                cost: edge.2,
            });
            let to_node = &mut nodes[to_node_id as usize];
            to_node.edges.push(Edge {
                to: from_node_id,
                cost: edge.2,
            });
        }
        Ok(Graph {
            node_count,
            nodes,
        })
    }

    fn find_node_id(&self, label: &str) -> Option<u32> {
        for node in &self.nodes {
            if node.label == label {
                return Some(node.id);
            }
        }
        None
    }

    fn find_longest_path(&self, root: u32) -> Path {
        let mut finder = LongestPathFinder::new(self);
        finder.get_longest_path(root)
    }
}

#[derive(Debug, Clone)]
struct Path {
    node_ids: Vec<u32>,
    distance_sum: u64,
}

impl Default for Path {
    fn default() -> Path {
        Path {
            node_ids: Vec::new(),
            distance_sum: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct LongestPathFinder<'a> {
    graph: &'a Graph,
    visited_nodes: HashSet<u32>,
    longest_paths: Vec<Path>,
}

impl LongestPathFinder<'_> {
    fn new(graph: &Graph) -> LongestPathFinder {
        LongestPathFinder {
            graph,
            visited_nodes: HashSet::new(),
            longest_paths: vec![Path::default(); graph.node_count as usize],
        }
    }

    fn calc_longest_path(&mut self, node_id: u32, path: Path) {
        if self.visited_nodes.contains(&node_id) {
            return;
        }
        let mut path = path;
        path.node_ids.push(node_id);
        self.visited_nodes.insert(node_id);
        if self.longest_paths[node_id as usize].distance_sum < path.distance_sum {
            self.longest_paths[node_id as usize] = path.clone();
        }

        let node = &self.graph.nodes[node_id as usize];
        for edge in &node.edges {
            let path = Path {
                node_ids: path.node_ids.clone(),
                distance_sum: path.distance_sum + edge.cost,
            };
            self.calc_longest_path(edge.to, path);
        }
        self.visited_nodes.remove(&node_id);
    }

    fn get_longest_path(&mut self, root: u32) -> Path {
        self.calc_longest_path(root, Path::default());
        let mut longest_path = Path::default();
        for path in &self.longest_paths {
            if longest_path.distance_sum < path.distance_sum {
                longest_path = path.clone();
            }
        }
        longest_path
    }
}

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
            eprintln!("Usage: {} <root node label>", std::env::args().nth(0).unwrap());
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
    println!("Longest path:");
    let longest_path = graph.find_longest_path(root);
    for node_id in &longest_path.node_ids {
        let node = &graph.nodes[*node_id as usize];
        println!("{}", node.label);
    }
    println!("Distance sum: {}", longest_path.distance_sum);
}
