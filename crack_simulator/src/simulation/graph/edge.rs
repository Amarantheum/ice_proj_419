use super::{node::{Node, NodeIndex}, NodeMatrix};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct EdgeIndex {
    pub row: usize,
    pub col: usize,
    pub ty: usize,
}

pub struct Edge {
    /// implicit stress in the edge
    pub imp_stress: f32,
    pub nodes: [NodeIndex; 2],

    pub index: EdgeIndex,
}

impl Edge {
    pub(super) fn new(imp_stress: f32, n1: NodeIndex, s1: usize, n2: NodeIndex, s2: usize, row: usize, col: usize, ty: usize, matrix: &mut NodeMatrix) -> Self {
        debug_assert!(ty < 3);
        let out = Self {
            imp_stress,
            nodes: [n1, n2],
            index: EdgeIndex { row, col, ty },
        };
        
        matrix.get_mut(n1).update_edge(s1, out.index);
        matrix.get_mut(n2).update_edge(s2, out.index);

        out
    }

    pub fn traverse(&self, n: NodeIndex) -> NodeIndex {
        if self.nodes[0] == n {
            self.nodes[1]
        } else {
            self.nodes[0]
        }
    }
}