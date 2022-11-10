use super::node::Node;

pub struct Edge<'a> {
    /// implicit stress in the edge
    pub imp_stress: f32,
    pub nodes: [&'a Node<'a>; 2],
    pub(super) valid: bool,

    row: usize,
    col: usize,
    ty: usize,
}

impl<'a> Edge<'a> {
    pub(super) fn new(imp_stress: f32, n1: &'a Node<'a>, s1: usize, n2: &'a Node<'a>, s2: usize, loc: *const Edge, row: usize, col: usize, ty: usize) -> Self {
        debug_assert!(ty < 3);
        let out = Self {
            imp_stress,
            nodes: [n1, n2],
            valid: true,
            row,
            col,
            ty,
        };
        unsafe {
            n1.get_mut_edges()[s1] = Some(&*loc);
            n2.get_mut_edges()[s2] = Some(&*loc);
        }
        out
    }

    #[allow(invalid_value)]
    pub(super) unsafe fn null() -> Self {
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