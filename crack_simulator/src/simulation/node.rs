use super::stress_vec::StressVec;
use super::edge::Edge;
pub struct Node {
    // the implicit stress in the node
    imp_stress: f32,
    stresses: Stresses,
    pub edges: [Option<*mut Edge>; 6],
}

impl Node {
    pub fn new(imp_stress: f32) -> Self {
        Self {
            imp_stress,
            edges: [None; 6],
            stresses: Stresses::default(),
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