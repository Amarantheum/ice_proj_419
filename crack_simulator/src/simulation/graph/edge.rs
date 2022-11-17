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

    // state
    total_stress: f32,
}

impl Edge {
    #[inline]
    pub(super) fn new(imp_stress: f32, n1: NodeIndex, s1: usize, n2: NodeIndex, s2: usize, row: usize, col: usize, ty: usize, matrix: &mut NodeMatrix) -> Self {
        debug_assert!(ty < 3);
        let out = Self {
            imp_stress,
            nodes: [n1, n2],
            index: EdgeIndex { row, col, ty },

            total_stress: 0.0,
        };
        
        matrix.get_mut(n1).update_edge(s1, out.index);
        matrix.get_mut(n2).update_edge(s2, out.index);

        out
    }

    #[inline]
    pub fn traverse(&self, n: NodeIndex) -> NodeIndex {
        if self.nodes[0] == n {
            self.nodes[1]
        } else {
            self.nodes[0]
        }
    }

    pub(super) fn update_total_stress(&mut self, matrix: &NodeMatrix) {
        let n1 = matrix.get(self.nodes[0]);
        let n2 = matrix.get(self.nodes[0]);
        self.total_stress = self.imp_stress
            + n1.imp_stress
            + n1.stresses.get_dir_stress(
                match self.index.ty {
                    0 => 0,
                    1 => 4,
                    2 => 5,
                    _ => unreachable!(),
                }
            )
            + n2.imp_stress
            + n2.stresses.get_dir_stress(
                match self.index.ty {
                    0 => 3,
                    1 => 1,
                    2 => 2,
                    _ => unreachable!(),
                }
            );
    }
}