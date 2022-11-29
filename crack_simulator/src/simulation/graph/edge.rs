use std::collections::VecDeque;

use super::{node::{Node, NodeIndex}, NodeMatrix, edge_update_list::EdgeUpdateList, EdgeMatrix, propagation_vector::PVec};
use super::stress_vec::StressVec;
use crate::graphics::vertex::Vertex;

use rand::random;

pub const CRACK_THRESHOLD: f32 = 1.9;
pub const PROPOGATION_CONST: f32 = 0.9;
const DIR_PROPAGATION: f32 = 10.0;

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
    pub nodes: [NodeIndex; 2],

    pub index: EdgeIndex,

    // state
    /// The total stress on the current edge
    /// from adjacent nodes. This value is also used
    /// when a crack occurs to propogate stress
    pub cracked: bool,
    update_status: EdgeUpdateStatus,
    pub prop_vec: PVec,
    prop_vec_update: PVec,
    pub stress: f32,
    stress_update: f32,
}

impl Edge {
    #[inline]
    pub(super) fn new(stress: f32, n1: NodeIndex, s1: usize, n2: NodeIndex, s2: usize, row: usize, col: usize, ty: usize, matrix: &mut NodeMatrix) -> Self {
        debug_assert!(ty < 3);
        let out = Self {
            stress,
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

    fn combine_stress(s: &mut f32, v: &mut PVec, stress: f32, dir: PVec, bias: f32) {
        if dir.is_zero() || stress == 0_f32 {
            return;
        }
        if v.is_zero() {
            *s += stress;
            *v = dir;
            return;
        }
        debug_assert!((v.modulus() * 100_f32).round() == 100.0);
        debug_assert!((dir.modulus() * 100_f32).round() == 100.0);
        *v = (v.scale(*s * bias) + dir.scale(stress / bias)).norm();
        *s += stress;
    }

    #[inline]
    pub fn add_stress_update(&mut self, stress: f32, dir: PVec) {
        Self::combine_stress(&mut self.stress_update, &mut self.prop_vec_update, stress, dir, 1.0);
    }

    /// Update self.stress and self.prop_vec with incomming updates
    #[inline]
    fn commit_updates(&mut self) {
        Self::combine_stress(&mut self.stress, &mut self.prop_vec, self.stress_update, self.prop_vec_update, 1.0);
        debug_assert!({
            if !((self.prop_vec.modulus() * 100_f32).round() == 100.0) && !self.prop_vec.is_zero() {
                println!("got bad mod: {}", self.prop_vec.modulus());
                println!("stress: {}, prop_vec: {:?}, stress_update: {}, prop_vec_update: {:?}", self.stress, self.prop_vec, self.stress_update, self.prop_vec_update);
                false
            } else {
                true
            }
        });
        self.stress_update = 0_f32;
        self.prop_vec_update = Default::default();
    }

    #[inline]
    pub(super) fn update_total_stress(&mut self, matrix: &mut NodeMatrix, update_list: &mut EdgeUpdateList) -> bool {
        if self.cracked {
            debug_assert!(self.stress == 0_f32);
            if self.stress_update != 0_f32 {
                // prepare self to propogate the stress
                self.commit_updates();
                self.set_scheduled_for_propagate_update();
                update_list.push(self.index);
            }
            return false;
        }
        self.commit_updates();

        if self.stress > CRACK_THRESHOLD {
            // edge is cracking
            self.cracked = true;
            
            if !self.prop_vec.is_zero() {
                let mut crack_adjustment = self.ty_to_prop_vec();
                if crack_adjustment * self.prop_vec < 0_f32 {
                    crack_adjustment = - crack_adjustment;
                }
                self.prop_vec = (self.prop_vec + crack_adjustment.scale(DIR_PROPAGATION * random::<f32>())).norm();
            }
            
            self.set_scheduled_for_propagate_update();
            update_list.push(self.index);
            true
        } else {
            self.set_not_scheduled_for_update();
            false
        }
    }

    /// Returns the edge indices of edges that surround the given edge without regard to the shape of the graph
    #[inline]
    pub(super) fn get_adjacent_edges(index: EdgeIndex, n_cols: usize) -> [Option<EdgeIndex>; 4] {
        match index.ty {
            0 => {
                let e1;
                let e2;
                let e3;
                let e4;
                if index.row == 0 {
                    e1 = None;
                    e2 = None;
                } else {
                    if index.row % 2 == 0 {
                        e1 = Some(EdgeIndex { row: index.row - 1, col: index.col, ty: 1});
                        e2 = Some(EdgeIndex { row: index.row - 1, col: index.col, ty: 2});
                    } else {
                        e1 = Some(EdgeIndex { row: index.row - 1, col: index.col + 1, ty: 1});
                        e2 = Some(EdgeIndex { row: index.row - 1, col: index.col + 1, ty: 2});
                    }
                }
                e3 = Some(EdgeIndex { row: index.row, col: index.col, ty: 2});
                e4 = Some(EdgeIndex { row: index.row, col: index.col + 1, ty: 1});
                [e1, e2, e3, e4]
            },
            1 => {
                let e1;
                let e2;
                let e3;
                let e4;
                if index.row % 2 == 1 && index.col == 0 {
                    e1 = None;
                    e2 = None;
                } else {
                    e1 = Some(EdgeIndex { row: index.row, col: index.col - 1, ty: 2});
                    e2 = Some(EdgeIndex { row: index.row, col: index.col - 1, ty: 0});
                }
                
                if index.row % 2 == 0 {
                    e3 = Some(EdgeIndex { row: index.row + 1, col: index.col - 1, ty: 0});
                } else {
                    e3 = Some(EdgeIndex { row: index.row + 1, col: index.col, ty: 0});
                }
                e4 = Some(EdgeIndex { row: index.row, col: index.col, ty: 2});
                [e1, e2, e3, e4]
            },
            2 => {
                let e1;
                let e2;
                let e3;
                let e4;

                e1 = Some(EdgeIndex { row: index.row, col: index.col, ty: 0});
                if index.col == n_cols - 1 {
                    e2 = None;
                } else {
                    e2 = Some(EdgeIndex { row: index.row, col: index.col + 1, ty: 1});
                }
                
                e3 = Some(EdgeIndex { row: index.row, col: index.col, ty: 1});
                if index.row % 2 == 0 {
                    if index.col == 0 {
                        e4 = None;
                    } else {
                        e4 = Some(EdgeIndex { row: index.row + 1, col: index.col - 1, ty: 0});
                    }       
                } else {
                    e4 = Some(EdgeIndex { row: index.row + 1, col: index.col, ty: 0});
                }
                [e1, e2, e3, e4]
            },
            _ => unreachable!(),
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

    pub fn ty_to_prop_vec(&self) -> PVec {
        match self.index.ty {
            0 => PVec::new(0.0, 1.0),
            1 => PVec::new(-3_f32.sqrt() / 2.0, 0.5),
            2 => PVec::new(3_f32.sqrt() / 2.0, 0.5),
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

    // pub(super) fn propogate_stress(&mut self, n_matrix: &mut NodeMatrix, orth_nodes: [Option<NodeIndex>; 2], e_update_list: &mut EdgeUpdateList) {
    //     if !self.cracked || self.total_stress == 0.0 {
    //         return;
    //     }
    //     for n in orth_nodes {
    //         if let Some(nn) = n {
    //             n_matrix.get_mut(nn).stresses.add_stress(self.ty_to_vec(self.total_stress * PROPOGATION_CONST));
    //         }
    //     }
    // }

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

    pub fn add_stress(&mut self, stress: f32, update_list: &mut EdgeUpdateList) {
        self.stress += stress;
        if self.update_status != EdgeUpdateStatus::StressUpdate {
            self.set_scheduled_for_stress_update();
            update_list.push(self.index);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_adjacent_edges() {
        let e = EdgeIndex { row: 0, col: 0, ty: 0};

        let mut e3 = e.clone();
        e3.ty = 2;

        let mut e4 = e.clone();
        e4.ty = 1;
        e4.col = 1;
        let o = Edge::get_adjacent_edges(e, 10);
        assert!(o[0].is_none());
        assert!(o[1].is_none());
        assert!(o[2].unwrap() == e3);
        assert!(o[3].unwrap() == e4);

        let e = EdgeIndex { row: 1, col: 1, ty: 1};
        let e1 = EdgeIndex {
            row: 1,
            col: 0,
            ty: 2,
        };
        let e2 = EdgeIndex {
            row: 1,
            col: 0,
            ty: 0
        };
        let e3 = EdgeIndex {
            row: 2,
            col: 1,
            ty: 0,
        };
        let e4 = EdgeIndex {
            row: 1,
            col: 1,
            ty: 2,
        };
        let o = Edge::get_adjacent_edges(e, 10);
        assert!(o[0].unwrap() == e1);
        assert!(o[1].unwrap() == e2);
        assert!(o[2].unwrap() == e3);
        assert!(o[3].unwrap() == e4);

        let e = EdgeIndex { row: 0, col: 1, ty: 1};
        let e1 = EdgeIndex {
            row: 0,
            col: 0,
            ty: 2,
        };
        let e2 = EdgeIndex {
            row: 0,
            col: 0,
            ty: 0
        };
        let e3 = EdgeIndex {
            row: 1,
            col: 0,
            ty: 0,
        };
        let e4 = EdgeIndex {
            row: 0,
            col: 1,
            ty: 2,
        };
        let o = Edge::get_adjacent_edges(e, 10);
        assert!(o[0].unwrap() == e1);
        assert!(o[1].unwrap() == e2);
        assert!(o[2].unwrap() == e3);
        assert!(o[3].unwrap() == e4);

        let e = EdgeIndex { row: 0, col: 1, ty: 2};
        let e1 = EdgeIndex {
            row: 0,
            col: 1,
            ty: 0,
        };
        let e2 = EdgeIndex {
            row: 0,
            col: 2,
            ty: 1,
        };
        let e3 = EdgeIndex {
            row: 0,
            col: 1,
            ty: 1,
        };
        let e4 = EdgeIndex {
            row: 1,
            col: 0,
            ty: 0,
        };
        let o = Edge::get_adjacent_edges(e, 10);
        assert!(o[0].unwrap() == e1);
        assert!(o[1].unwrap() == e2);
        assert!(o[2].unwrap() == e3);
        assert!(o[3].unwrap() == e4);

        let e = EdgeIndex { row: 1, col: 1, ty: 2};
        let e1 = EdgeIndex {
            row: 1,
            col: 1,
            ty: 0,
        };
        let e2 = EdgeIndex {
            row: 1,
            col: 2,
            ty: 1,
        };
        let e3 = EdgeIndex {
            row: 1,
            col: 1,
            ty: 1,
        };
        let e4 = EdgeIndex {
            row: 2,
            col: 1,
            ty: 0,
        };
        let o = Edge::get_adjacent_edges(e, 10);
        assert!(o[0].unwrap() == e1);
        assert!(o[1].unwrap() == e2);
        assert!(o[2].unwrap() == e3);
        assert!(o[3].unwrap() == e4);
    }
}