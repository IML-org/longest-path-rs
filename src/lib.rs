
#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EdgeData (String, String, u64);

#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub to: u32,
    pub cost: u64,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: u32,
    pub edges: Vec<Edge>,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct Graph {
    pub node_count: u32,
    pub nodes: Vec<Node>,
}

impl Graph {
    pub fn from_json_directed(json: &str) -> Result<Graph, serde_json::Error> {
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

    pub fn from_json_undirected(json: &str) -> Result<Graph, serde_json::Error> {
        let data: Vec<EdgeData> = serde_json::from_str(json)?;
        let edge_count = data.len();
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
        eprintln!("Graph imported: {} nodes, {} edges", node_count, edge_count);
        Ok(Graph {
            node_count,
            nodes,
        })
    }

    pub fn find_node_id(&self, label: &str) -> Option<u32> {
        for node in &self.nodes {
            if node.label == label {
                return Some(node.id);
            }
        }
        None
    }

    pub fn find_neighbor_id_set(&self, node_id: u32) -> HashSet<u32> {
        let mut neighbor_id_set = HashSet::new();
        let node = &self.nodes[node_id as usize];
        for edge in &node.edges {
            neighbor_id_set.insert(edge.to);
        }
        neighbor_id_set
    }

    pub fn find_longest_path(&self, root: u32) -> Path {
        let mut finder = LongestPathFinder::new(self);
        finder.get_longest_path(root)
    }

    pub fn find_longest_path_to(&self, root: u32, to_node_id: u32) -> Path {
        let mut finder = LongestPathFinder::new(self);
        finder.get_longest_path_to(root, to_node_id)
    }

    pub fn find_longest_path_to_paralell(&self, root: u32, to_node_id: u32) -> Path {
        let finder = LongestPathFinderParalell::new(self, root, to_node_id);
        finder.get_longest_path_to()
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    pub node_ids: Vec<u32>,
    pub node_id_set: HashSet<u32>,
    pub distance_sum: u64,
}

impl Default for Path {
    fn default() -> Path {
        Path {
            node_ids: Vec::new(),
            node_id_set: HashSet::new(),
            distance_sum: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct LongestPathFinder<'a> {
    graph: &'a Graph,
    visited_nodes: HashSet<u32>,
    longest_paths: Vec<Path>,
    found_path_count: u64,
}

impl LongestPathFinder<'_> {
    fn new(graph: &Graph) -> LongestPathFinder {
        LongestPathFinder {
            graph,
            visited_nodes: HashSet::new(),
            longest_paths: vec![Path::default(); graph.node_count as usize],
            found_path_count: 0,
        }
    }

    fn calc_longest_path(&mut self, node_id: u32, path: Path) {
        if self.visited_nodes.contains(&node_id) {
            return;
        }
        let mut path = path;
        path.node_ids.push(node_id);
        path.node_id_set.insert(node_id);
        self.visited_nodes.insert(node_id);
        if self.longest_paths[node_id as usize].distance_sum < path.distance_sum {
            self.longest_paths[node_id as usize] = path.clone();
        }

        let node = &self.graph.nodes[node_id as usize];
        for edge in &node.edges {
            let path = Path {
                node_ids: path.node_ids.clone(),
                node_id_set: path.node_id_set.clone(),
                distance_sum: path.distance_sum + edge.cost,
            };
            self.calc_longest_path(edge.to, path);
        }
        self.visited_nodes.remove(&node_id);
    }

    fn calc_longest_path_to(&mut self, node_id: u32, to_node_id: u32, mut path: Path, target_neighbor_id_set: &HashSet<u32>) -> (Path,) {
        path.node_ids.push(node_id);
        path.node_id_set.insert(node_id);

        let finished = path.node_id_set.is_superset(target_neighbor_id_set);

        if node_id == to_node_id {
            if self.longest_paths[node_id as usize].distance_sum < path.distance_sum {
                self.longest_paths[node_id as usize] = path.clone();
            }
            self.found_path_count += 1;
            if self.found_path_count % 1000000 == 0 {
                let longest = self.longest_paths[node_id as usize].distance_sum;
                eprintln!("Found {} paths, longest: {}", self.found_path_count, longest);
            }
        } else {
            let node = &self.graph.nodes[node_id as usize];
            for edge in &node.edges {
                if finished && edge.to != to_node_id {
                    continue;
                }
                if path.node_id_set.contains(&edge.to) {
                    continue;
                }
                path.distance_sum += edge.cost;
                (path,) = self.calc_longest_path_to(edge.to, to_node_id, path, target_neighbor_id_set);
                path.distance_sum -= edge.cost;
            }
        }

        path.node_id_set.remove(&node_id);
        path.node_ids.pop();
        (path,)
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

    fn get_longest_path_to(&mut self, root: u32, to_node_id: u32) -> Path {
        let target_neighbor_id_set = self.graph.find_neighbor_id_set(to_node_id);
        self.calc_longest_path_to(root, to_node_id, Path::default(), &target_neighbor_id_set);
        self.longest_paths[to_node_id as usize].clone()
    }
}

#[derive(Debug)]
struct LongestPathFinderParalell<'a> {
    graph: &'a Graph,
    from_node_id: u32,
    to_node_id: u32,
    longest_path_distance: AtomicU64,
    found_path_count: AtomicU64,
    target_neighbor_id_set: HashSet<u32>,
}

impl<'main> LongestPathFinderParalell<'main> {
    fn new(graph: &Graph, from_node_id: u32, to_node_id: u32) -> Arc<LongestPathFinderParalell> {
        Arc::new(LongestPathFinderParalell {
            graph,
            from_node_id,
            to_node_id,
            longest_path_distance: AtomicU64::new(0),
            found_path_count: AtomicU64::new(0),
            target_neighbor_id_set: graph.find_neighbor_id_set(to_node_id),
        })
    }

    fn calc_longest_path_to<'a>(self: Arc<Self>, node_id: u32, to_node_id: u32, mut path: Path, found_path_sender: mpsc::Sender<Path>, scope: &rayon::Scope<'a>)
    where 'main: 'a
    {
        path.node_ids.push(node_id);
        path.node_id_set.insert(node_id);

        let finished = path.node_id_set.is_superset(&self.target_neighbor_id_set);

        if node_id == to_node_id {
            let count = self.found_path_count.fetch_add(1, Ordering::SeqCst);
            let count = count + 1;
            let prev = self.longest_path_distance.fetch_max(path.distance_sum, Ordering::SeqCst);
            if prev < path.distance_sum {
                found_path_sender.send(path.clone()).unwrap();
            }
            let longest = if prev < path.distance_sum { path.distance_sum } else { prev };
            
            eprintln!("Found {} paths, longest: {}", count, longest);
            return;
        }

        let node = &self.graph.nodes[node_id as usize];
        for edge in &node.edges {
            if finished && edge.to != to_node_id {
                continue;
            }
            if path.node_id_set.contains(&edge.to) {
                continue;
            }
            let path = Path {
                node_ids: path.node_ids.clone(),
                node_id_set: path.node_id_set.clone(),
                distance_sum: path.distance_sum + edge.cost,
            };
            let found_path_sender = found_path_sender.clone();
            let node_id = edge.to;
            let self_clone = self.clone();
            scope.spawn(move |s| {
                self_clone.calc_longest_path_to(node_id, to_node_id, path, found_path_sender, s);
            });
        }
    }

    fn get_longest_path_to(self: Arc<Self>) -> Path {
        let root = self.from_node_id;
        let to_node_id = self.to_node_id;
        let (tx, rx) = mpsc::channel();
        let self_clone = self.clone();
        let longest_path = Arc::new(Mutex::new(Path::default()));
        let longest_path_clone = longest_path.clone();
        thread::scope(|s| {
            s.spawn(|| {
                rayon::scope(|s| {
                    self_clone.calc_longest_path_to(root, to_node_id, Path::default(), tx, s);
                });
            });

            s.spawn(move || {
                let mut longest_path = longest_path_clone.lock().unwrap();
                // let mut update_count = 0_u64;
                while let Ok(path) = rx.recv() {
                    if longest_path.distance_sum < path.distance_sum {
                        *longest_path = path;
                        // update_count += 1;
                        // let current_longest_path_distance = longest_path.distance_sum;
                        // eprint!("Found {} paths, current longest path: {}\r", update_count, current_longest_path_distance);
                        // if update_count % 1000 == 0 {
                        //     eprintln!("Found {} paths", update_count);
                        // }
                    } else {
                        eprintln!("This should not happen!");
                    }
                }
            });
        });
        let longest_path = longest_path.lock().unwrap();
        (*longest_path).clone()
    }
}
