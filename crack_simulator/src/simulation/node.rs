use super::stress_vec::StressVec;
use super::edge::Edge;

pub struct Node<'a> {
    // the implicit stress in the node
    pub imp_stress: f32,
    stresses: Stresses,
    pub edges: [Option<&'a Edge<'a>>; 6],
}

impl<'a> Node<'a> {
    pub fn new(imp_stress: f32) -> Self {
        Self {
            imp_stress,
            edges: [None; 6],
            stresses: Stresses::default(),
        }
    }
    
    /// Returns a mutable pointer to this node's edges from an immutable pointer
    pub unsafe fn get_mut_edges<'b>(&self) -> &'b mut [Option<&'a Edge<'a>>; 6] {
        &mut *(&(self.edges) as *const [Option<&'a Edge<'a>>; 6] as *mut [Option<&'a Edge<'a>>; 6])
    }

    /// Get the node adjacent to this node in direction n (n=0 => 0rad, n=1 => pi/3rad ...)
    pub fn get_adjacent_node_n(&'a self, n: usize) -> Option<&'a Node<'a>> {
        debug_assert!(n < 6);
        match self.edges[n] {
            Some(e) => Some(e.traverse(&self)),
            None => None,
        }
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