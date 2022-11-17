use std::collections::HashSet;

use super::{NodeMatrix, EdgeMatrix};
use super::stress_vec::StressVec;
use super::edge::{Edge, EdgeIndex};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct NodeIndex {
    pub row: usize,
    pub col: usize,
}

impl From<[usize; 2]> for NodeIndex {
    fn from(v: [usize; 2]) -> Self {
        Self {
            row: v[0],
            col: v[1],
        }
    }
}

pub struct Node {
    // the implicit stress in the node
    pub imp_stress: f32,
    pub stresses: Stresses,
    pub edges: [Option<EdgeIndex>; 6],
    
    pub index: NodeIndex,
}

impl Node {
    pub(super) fn new(imp_stress: f32, row: usize, col: usize) -> Self {
        Self {
            imp_stress,
            edges: [None; 6],
            stresses: Stresses::default(),
            index: NodeIndex { row, col }
        }
    }

    /// Get the node adjacent to this node in direction n (n=0 => 0rad, n=1 => pi/3rad ...)
    pub fn get_adjacent_node_n(&self, n: usize, matrix: &EdgeMatrix) -> Option<NodeIndex> {
        debug_assert!(n < 6);
        match self.edges[n] {
            Some(e) => {
                let edge = matrix.get(e)
                    .expect("this should never fail...");
                Some(edge.traverse(self.index))
            },
            None => None,
        }
    }

    pub(super) fn update_edge(&mut self, edge: usize, i: EdgeIndex) {
        self.edges[edge] = Some(i)
    }

    pub fn verify(&self, n_matrix: &NodeMatrix, e_matrix: &EdgeMatrix, rows: usize, cols: usize, index: NodeIndex) {
        assert!(index == self.index);
        let mut req_edges = HashSet::new();
        for i in 0..6_usize {
            req_edges.insert(i);
        }

        if self.index.col == 0 {
            req_edges.remove(&1);
            req_edges.remove(&2);
        }

        if self.index.col == cols - 1 {
            req_edges.remove(&4);
            req_edges.remove(&5);
        }

        if self.index.row == 0 {
            if self.index.row % 2 == 0 {
                req_edges.remove(&2);
                req_edges.remove(&4);
            }
            req_edges.remove(&3);
        }

        if self.index.row == rows - 1 {
            if self.index.row % 2 == 1 {
                req_edges.remove(&1);
                req_edges.remove(&5);
            }
            req_edges.remove(&0);
        }

        for e in req_edges {
            let edge = e_matrix.get(self.edges[e].expect("edge shouldn't be None")).expect("edge shouldn't be None");
            match e {
                0 | 4 | 5 => assert!(edge.nodes[0] == self.index),
                1 | 2 | 3 => assert!(edge.nodes[1] == self.index),
                _ => unreachable!(),
            }
            //edge.verify(n_matrix)
        }
    }
}

pub struct Stresses {
    right: f32,
    up: f32,
    left: f32,
    down: f32,
}

impl Stresses {
    fn new(right: f32, up: f32, left: f32, down: f32) -> Self {
        Self {
            right,
            up,
            left,
            down,
        }
    }

    fn add_stress(s: StressVec) {
        match s {
            StressVec::A0(f) => {

            },
            StressVec::A1(f) => {

            },
            StressVec::A2(f) => {

            }
        }
    }

    pub fn get_dir_stress(&self, n: usize) -> f32 {
        debug_assert!(n < 6);
        match n {
            0 => self.right,
            1 => self.right / 2.0 + self.up * 3_f32.sqrt() / 2.0,
            2 => self.left / 2.0 + self.up * 3_f32.sqrt() / 2.0,
            3 => self.left,
            4 => self.left / 2.0 + self.down * 3_f32.sqrt() / 2.0,
            5 => self.right / 2.0 + self.down * 3_f32.sqrt() / 2.0,
            _ => panic!("Dir stress is not defined for undefined directions"),
        }
    }
}

impl Default for Stresses {
    fn default() -> Self {
        Self::new(0_f32, 0_f32, 0_f32, 0_f32)
    }
}