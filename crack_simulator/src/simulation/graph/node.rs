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
    stresses: Stresses,
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
}

struct Stresses {
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
}

impl Default for Stresses {
    fn default() -> Self {
        Self::new(0_f32, 0_f32, 0_f32, 0_f32)
    }
}