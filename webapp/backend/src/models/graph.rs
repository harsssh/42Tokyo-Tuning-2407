use sqlx::FromRow;
use std::collections::{HashMap, HashSet};
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

    // return node_id
    pub fn find_closest_node(
        &self,
        from_node_id: i32,
        to_node_ids: Vec<i32>,
        limit: i32,
    ) -> Option<i32> {
        let goals: HashSet<_> = to_node_ids.iter().collect();

        let mut distances = HashMap::new();
        let mut heap = BinaryHeap::new();

        heap.push(Reverse((0, from_node_id)));

        while let Some(Reverse((distance, node_id))) = heap.pop() {
            if goals.contains(&node_id) {
                return Some(node_id);
            }

            if distance > *distances.get(&node_id).unwrap_or(&limit) {
                continue;
            }

            distances.insert(node_id, distance);

            if let Some(edges) = self.edges.get(&node_id) {
                for edge in edges.iter() {
                    let new_distance = distance + edge.weight;
                    if new_distance <= *distances.get(&edge.node_b_id).unwrap_or(&limit) {
                        heap.push(Reverse((new_distance, edge.node_b_id)));
                    }
                }
            }
        }

        None
    }
}
