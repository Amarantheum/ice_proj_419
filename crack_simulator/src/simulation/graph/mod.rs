use node::Node;
use edge::Edge;

pub mod node;
pub mod edge;
mod stress_vec;

/// Graph of stress nodes
/// Top left corner starts like:
/// * * * *
///  * * * *
/// * * * *
///  * * * * 
pub struct Graph<'a> {
    /// number of rows in graph
    rows: usize,
    /// number of columns in graph
    cols: usize,
    /// matrix of nodes
    node_matrix: Box<[Node<'a>]>,
    /// matrix of edges. Note there's 3 types of edges at each level
    /// 0 => -, 1 => /, 2 => \
    edge_matrix: Box<[[Edge<'a>; 3]]>,
}

impl<'a> Graph<'a> {
    /// rows = number of rows (y axis), cols = num calls (x axis)
    #[inline]
    pub fn new(rows: usize, cols: usize) -> Self {

        let mut node_matrix_v = Vec::with_capacity(rows * cols);
        unsafe {
            node_matrix_v.set_len(rows * cols);
        }
        let node_matrix_p = Box::into_raw(node_matrix_v.into_boxed_slice());

        let mut edge_matrix_v = Vec::with_capacity(rows * cols);
        unsafe {
            edge_matrix_v.set_len(rows * cols);
        }
        let edge_matrix_p = Box::into_raw(edge_matrix_v.into_boxed_slice());

        unsafe { 
            Self::init(rows, cols, node_matrix_p, edge_matrix_p);
            Self {
                rows,
                cols,
                node_matrix: Box::from_raw(node_matrix_p),
                edge_matrix: Box::from_raw(edge_matrix_p),
            }
        }
    }
    #[inline]
    unsafe fn init<'b>(rows: usize, cols: usize, node_matrix: *mut [Node<'b>], edge_matrix: *mut [[Edge<'b>; 3]]) {
        // initialize node matrix
        for i in 0..(rows * cols) {
            (*node_matrix)[i] = Node::new(Self::get_init_implicit_node_stress(), i / cols, i % cols);
        }

        let get_node = |x: usize, y: usize| -> &Node {
            &(*node_matrix)[y * cols + x]
        };

        for y in 0..rows {
            for x in 0..cols {
                // fill in horizontal edges for this row
                if x < cols - 1 {
                    // if we aren't on the last col, link this node to the node adjacent to the right
                    (*edge_matrix)[y * cols + x][0] = Edge::new(Self::get_init_implicit_edge_stress(), get_node(x, y), 0, get_node(x + 1, y), 3, &(*edge_matrix)[y * cols + x][0], y, x, 0);
                } else {
                    (*edge_matrix)[y * cols + x][0] = Edge::null();
                }

                if y < rows - 1 {
                    // if we're not on the last row
                    if y % 2 != 1 {
                        // if we're on an even row (including row 0)
                        if x > 0 {
                            (*edge_matrix)[y * cols + x][1] = Edge::new(Self::get_init_implicit_edge_stress(), get_node(x, y), 4, get_node(x - 1, y + 1), 1, &(*edge_matrix)[y * cols + x][1], y, x, 1);
                        } else {
                            (*edge_matrix)[y * cols + x][1] = Edge::null();
                        }
                        (*edge_matrix)[y * cols + x][2] = Edge::new(Self::get_init_implicit_edge_stress(), get_node(x, y), 5, get_node(x, y + 1), 2, &(*edge_matrix)[y * cols + x][2], y, x, 2);
                    } else {
                        (*edge_matrix)[y * cols + x][1] = Edge::new(Self::get_init_implicit_edge_stress(), get_node(x, y), 4, get_node(x, y + 1), 1, &(*edge_matrix)[y * cols + x][1], y, x, 1);
                        if x < cols - 1 {
                            (*edge_matrix)[y * cols + x][2] = Edge::new(Self::get_init_implicit_edge_stress(), get_node(x, y), 5, get_node(x + 1, y + 1), 2, &(*edge_matrix)[y * cols + x][2], y, x, 2);
                        } else {
                            (*edge_matrix)[y * cols + x][2] = Edge::null();
                        }
                    }
                } else {
                    // if we're on the last row
                    (*edge_matrix)[y * cols + x][1] = Edge::null();
                    (*edge_matrix)[y * cols + x][2] = Edge::null();
                }
            }
           
            //self.edge_matrix.push(cur_col);
        }
    }

    #[inline]
    pub fn get_node(&'a self, x: usize, y: usize) -> &'a Node<'a> {
        &self.node_matrix[y * self.cols + x]
    }

    #[inline]
    pub fn get_node_mut(&'a mut self, x: usize, y: usize) -> &'a mut Node<'a> {
        &mut self.node_matrix[y * self.cols + x]
    }

    #[inline]
    pub fn get_edge(&'a self, x: usize, y: usize, t: usize) -> Option<&'a Edge<'a>> {
        if self.edge_matrix[y * self.cols + x][t].valid {
            Some(&self.edge_matrix[y * self.cols + x][t])
        } else {
            None
        }
    }
    
    #[inline]
    pub fn get_edge_mut(&'a mut self, x: usize, y: usize, t: usize) -> Option<&'a mut Edge<'a>> {
        if self.edge_matrix[y * self.cols + x][t].valid {
            Some(&mut self.edge_matrix[y * self.cols + x][t])
        } else {
            None
        }
        
    }

    fn get_init_implicit_node_stress() -> f32 {
        // TODO randomize here?
        0_f32
    }

    fn get_init_implicit_edge_stress() -> f32 {
        // TODO randomize here?
        0_f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;

    #[test]
    fn test_random_traverse() {
        let g = Graph::new(1000, 1000);

        let mut cur_node = g.get_node(0, 0);

        for _ in 0..100000 {
            let mut n = random::<usize>() % 6;
            loop {
                if let Some(v) = cur_node.get_adjacent_node_n(n) {
                    cur_node = v;
                    break;
                } else {
                    n += 1;
                    n %= 6;
                }
            }
        }
    }
}