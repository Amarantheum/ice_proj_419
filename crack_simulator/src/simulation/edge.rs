use super::node::Node;

pub struct Edge<'a> {
    /// implicit stress in the edge
    pub imp_stress: f32,
    nodes: [&'a Node<'a>; 2],
    pub valid: bool,
}

impl<'a> Edge<'a> {
    pub fn new(imp_stress: f32, n1: &'a Node<'a>, s1: usize, n2: &'a Node<'a>, s2: usize, loc: *const Edge) -> Self {
        let out = Self {
            imp_stress,
            nodes: [n1, n2],
            valid: true,
        };
        unsafe {
            n1.get_mut_edges()[s1] = Some(&*loc);
            n2.get_mut_edges()[s2] = Some(&*loc);
        }
        out
    }

    pub unsafe fn null() -> Self {
        let mut out: Self = std::mem::MaybeUninit::uninit().assume_init();
        out.valid = false;
        out
    }

    pub fn traverse(&'a self, n: &'a Node<'a>) -> &'a Node<'a> {
        if self.nodes[0] as *const Node == n as *const Node {
            self.nodes[1]
        } else {
            self.nodes[0]
        }
    }
}