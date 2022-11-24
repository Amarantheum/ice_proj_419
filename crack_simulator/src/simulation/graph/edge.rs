use std::collections::VecDeque;

use super::{node::{Node, NodeIndex}, NodeMatrix, edge_update_list::EdgeUpdateList, EdgeMatrix};
use super::stress_vec::StressVec;

const CRACK_THRESHOLD: f32 = 1.5;
const PROPOGATION_CONST: f32 = 1.0;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct EdgeIndex {
    pub row: usize,
    pub col: usize,
    pub ty: usize,
}

#[derive(PartialEq, Clone, Copy)]
pub enum EdgeUpdateStatus {
    NoUpdate,
    StressUpdate,
    PropogationUpdate,
}

impl Default for EdgeUpdateStatus {
    fn default() -> Self {
        Self::NoUpdate
    }
}

#[derive(Default)]
pub struct Edge {
    /// implicit stress in the edge
    pub imp_stress: f32,
    pub nodes: [NodeIndex; 2],

    pub index: EdgeIndex,

    // state
    /// The total stress on the current edge
    /// from adjacent nodes. This value is also used
    /// when a crack occurs to propogate stress
    total_stress: f32,
    cracked: bool,
    update_status: EdgeUpdateStatus,
}

impl Edge {
    #[inline]
    pub(super) fn new(imp_stress: f32, n1: NodeIndex, s1: usize, n2: NodeIndex, s2: usize, row: usize, col: usize, ty: usize, matrix: &mut NodeMatrix) -> Self {
        debug_assert!(ty < 3);
        let out = Self {
            imp_stress,
            nodes: [n1, n2],
            index: EdgeIndex { row, col, ty },
            ..Default::default()
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

    #[inline]
    pub(super) fn update_total_stress(&mut self, matrix: &NodeMatrix, update_list: &mut EdgeUpdateList) {
        if self.cracked { return }
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
        if self.total_stress > CRACK_THRESHOLD {
            self.cracked = true;
            self.set_scheduled_for_propagate_update();
            update_list.push(self.index);
        } else {
            self.set_not_scheduled_for_update();
        }
    }

    #[inline]
    pub(super) fn get_orthogonal_nodes(&self, n_matrix: &NodeMatrix, e_matrix: &EdgeMatrix) -> [Option<NodeIndex>; 2] {
        match self.index.ty {
            0 => {
                let upper_node = n_matrix.get(self.nodes[0]).get_adjacent_node_n(1, e_matrix);
                let lower_node = n_matrix.get(self.nodes[0]).get_adjacent_node_n(5, e_matrix);
                [upper_node, lower_node]
            },
            1 => {
                let upper_node = n_matrix.get(self.nodes[0]).get_adjacent_node_n(3, e_matrix);
                let lower_node = n_matrix.get(self.nodes[0]).get_adjacent_node_n(5, e_matrix);
                [upper_node, lower_node]
            },
            2 => {
                let upper_node = n_matrix.get(self.nodes[0]).get_adjacent_node_n(0, e_matrix);
                let lower_node = n_matrix.get(self.nodes[0]).get_adjacent_node_n(4, e_matrix);
                [upper_node, lower_node]
            },
            _ => unreachable!(),
        }
    }

    fn ty_to_vec(&self, stress: f32) -> StressVec {
        match self.index.ty {
            0 | 3 => StressVec::A0(stress),
            1 | 4 => StressVec::A1(stress),
            2 | 5 => StressVec::A2(stress),
            _ => unreachable!(),
        }
    }

    pub(super) fn propogate_stress(&mut self, n_matrix: &mut NodeMatrix, orth_nodes: [Option<NodeIndex>; 2], e_update_list: &mut EdgeUpdateList) {
        if !self.cracked || self.total_stress == 0.0 {
            return;
        }
        for n in orth_nodes {
            if let Some(nn) = n {
                n_matrix.get_mut(nn).stresses.add_stress(self.ty_to_vec(self.total_stress * PROPOGATION_CONST));
            }
        }
    }

    pub fn verify(&self, n_matrix: &NodeMatrix) {
        let indexes;
        match self.index.ty {
            0 => indexes = [0, 3],
            1 => indexes = [4, 1],
            2 => indexes = [5, 2],
            _ => unreachable!(),

        }

        let n1 = n_matrix.get(self.nodes[0]);
        let n2 = n_matrix.get(self.nodes[1]);

        assert!(n1.edges[indexes[0]].expect("Shouldn't be None") == self.index);
        assert!(n2.edges[indexes[1]].expect("Shouldn't be None") == self.index);
    }

    pub fn set_scheduled_for_stress_update(&mut self) {
        self.update_status = EdgeUpdateStatus::StressUpdate;
    }

    pub fn set_scheduled_for_propagate_update(&mut self) {
        self.update_status = EdgeUpdateStatus::PropogationUpdate;
    }

    pub fn set_not_scheduled_for_update(&mut self) {
        self.update_status = EdgeUpdateStatus::NoUpdate;
    }

    pub fn set_update_status_propogated(&mut self) {
        if self.update_status == EdgeUpdateStatus::PropogationUpdate {
            self.update_status = EdgeUpdateStatus::NoUpdate;
        }
    }

    pub fn get_update_status(&self) -> EdgeUpdateStatus {
        self.update_status
    }
}