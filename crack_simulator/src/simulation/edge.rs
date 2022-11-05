use super::node::Node;

pub struct Edge {
    /// implicit stress in the edge
    imp_stress: f32,
    nodes: [*mut Node; 2],
}

impl Edge {
    pub fn new(imp_stress: f32, n1: *mut Node, s1: usize, n2: *mut Node, s2: usize) -> Self {
        let mut out = Self {
            imp_stress,
            nodes: [n1 as *mut Node, n2 as *mut Node],
        };
        n1.edges[s1] = Some(&mut out as *mut Edge);
        n2.edges[s2] = Some(&mut out as *mut Edge);
        out
    }
}