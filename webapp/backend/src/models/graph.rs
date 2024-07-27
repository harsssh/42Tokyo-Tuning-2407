use sqlx::FromRow;
use std::collections::HashMap;
use std::{cmp::Reverse, collections::BinaryHeap};

#[derive(FromRow, Clone, Debug)]
pub struct Node {
    pub id: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(FromRow, Clone, Debug)]
pub struct Edge {
    pub node_a_id: i32,
    pub node_b_id: i32,
    pub weight: i32,
}

#[derive(Debug)]
pub struct Graph {
    pub nodes: HashMap<i32, Node>,
    pub edges: HashMap<i32, Vec<Edge>>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges
            .entry(edge.node_a_id)
            .or_default()
            .push(edge.clone());

        let reverse_edge = Edge {
            node_a_id: edge.node_b_id,
            node_b_id: edge.node_a_id,
            weight: edge.weight,
        };
        self.edges
            .entry(reverse_edge.node_a_id)
            .or_default()
            .push(reverse_edge);
    }

    pub fn shortest_path(&self, from_node_id: i32, to_node_id: i32) -> i32 {
        let mut distances = vec![std::i32::MAX / 2; self.nodes.len() + 1];
        let mut heap = BinaryHeap::new();

        heap.push(Reverse((0, from_node_id)));

        while let Some(Reverse((distance, node_id))) = heap.pop() {
            if node_id == to_node_id {
                return distance;
            }

            if distance >= distances[node_id as usize] {
                continue;
            }

            distances[node_id as usize] = distance;

            if let Some(edges) = self.edges.get(&node_id) {
                for edge in edges.iter() {
                    if distance + edge.weight < distances[edge.node_b_id as usize] {
                        heap.push(Reverse((distance + edge.weight, edge.node_b_id)));
                    }
                }
            }
        }

        i32::MAX
    }
}
