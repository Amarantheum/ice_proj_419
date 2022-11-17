use node::Node;
use edge::Edge;
use rayon::{slice::{ParallelSlice, ParallelSliceMut}, current_num_threads};

use self::node::NodeIndex;
use self::edge::EdgeIndex;

use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

pub mod node;
pub mod edge;
mod stress_vec;

pub struct NodeMatrix {
    v: Vec<Vec<Node>>,
}

pub struct EdgeMatrix {
    v: Vec<Vec<[Option<Edge>; 3]>>
}

/// Graph of stress nodes
/// Top left corner starts like:
/// * * * *
///  * * * *
/// * * * *
///  * * * * 
pub struct Graph {
    /// number of rows in graph
    rows: usize,
    /// number of columns in graph
    cols: usize,
    /// matrix of nodes
    node_matrix: NodeMatrix,
    /// matrix of edges. Note there's 3 types of edges at each level
    /// 0 => -, 1 => /, 2 => \
    edge_matrix: EdgeMatrix,
}

impl Graph {
    /// rows = number of rows (y axis), cols = num calls (x axis)
    #[inline]
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut out = Self {
            rows,
            cols,
            node_matrix: NodeMatrix { v: Vec::with_capacity(rows) },
            edge_matrix: EdgeMatrix { v: Vec::with_capacity(rows) },
        };
        out.init();
        out
    }
    #[inline]
    fn init<'b>(&mut self) {
        // initialize node matrix
        for r in 0..self.rows {
            let mut cur = Vec::with_capacity(self.cols);
            for c in 0..self.cols {
                cur.push(Node::new(Self::get_init_implicit_node_stress(), r, c))
            }
            self.node_matrix.v.push(cur);
        }

        for y in 0..self.rows {
            let mut cur_vec: Vec<[Option<Edge>; 3]> = Vec::with_capacity(self.cols);
            for x in 0..self.cols {
                let mut cur = [None, None, None];
                // fill in horizontal edges for this row
                if x < self.cols - 1 {
                    // if we aren't on the last col, link this node to the node adjacent to the right
                    cur[0] = Some(Edge::new(Self::get_init_implicit_edge_stress(), [y, x].into(), 0, [y, x + 1].into(), 3, y, x, 0, &mut self.node_matrix));
                } else {
                    cur[0] = None;
                }

                if y < self.rows - 1 {
                    // if we're not on the last row
                    if y % 2 != 1 {
                        // if we're on an even row (including row 0)
                        if x > 0 {
                            cur[1] = Some(Edge::new(Self::get_init_implicit_edge_stress(), [y, x].into(), 4, [y + 1, x - 1].into(), 1, y, x, 1, &mut self.node_matrix));
                        } else {
                            cur[1] = None;
                        }
                        cur[2] = Some(Edge::new(Self::get_init_implicit_edge_stress(), [y, x].into(), 5, [y + 1, x].into(), 2, y, x, 2, &mut self.node_matrix));
                    } else {
                        cur[1] = Some(Edge::new(Self::get_init_implicit_edge_stress(), [y, x].into(), 4, [y + 1, x].into(), 1, y, x, 1, &mut self.node_matrix));
                        if x < self.cols - 1 {
                            cur[2] = Some(Edge::new(Self::get_init_implicit_edge_stress(), [y, x].into(), 5, [y + 1, x + 1].into(), 2, y, x, 2, &mut self.node_matrix));
                        } else {
                            cur[2] = None;
                        }
                    }
                } else {
                    // if we're on the last row
                    cur[1] = None;
                    cur[2] = None;
                }
                cur_vec.push(cur);
            }
            self.edge_matrix.v.push(cur_vec);
        }
    }

    #[inline]
    pub fn get_node(&self, i: NodeIndex) -> &Node {
        self.node_matrix.get(i)
    }

    #[inline]
    pub fn get_node_mut(&mut self, i: NodeIndex) -> &mut Node {
        self.node_matrix.get_mut(i)
    }

    #[inline]
    pub fn get_edge(&self, x: usize, y: usize, t: usize) -> Option<&Edge> {
        self.edge_matrix.v[y][x][t].as_ref()
    }
    
    #[inline]
    pub fn get_edge_mut(&mut self, x: usize, y: usize, t: usize) -> Option<&mut Edge> {
        self.edge_matrix.v[y][x][t].as_mut()
    }

    fn get_init_implicit_node_stress() -> f32 {
        // TODO randomize here?
        0_f32
    }

    fn get_init_implicit_edge_stress() -> f32 {
        // TODO randomize here?
        0_f32
    }

    pub fn update_graph_edge_stresses(&mut self) {
        self.edge_matrix.v.par_iter_mut().for_each(|v| {
            for vv in v {
                for i in 0..3 {
                    if let Some(e) = vv[i].as_mut() {
                        e.update_total_stress(&self.node_matrix);
                    }
                }
            }
        });
    }
}

impl NodeMatrix {
    pub fn get(&self, i: NodeIndex) -> &Node {
        &self.v[i.row][i.col]
    }

    pub fn get_mut(&mut self, i: NodeIndex) -> &mut Node {
        &mut self.v[i.row][i.col]
    }
}

impl EdgeMatrix {
    pub fn get(&self, i: EdgeIndex) -> Option<&Edge> {
        self.v[i.row][i.col][i.ty].as_ref()
    }

    pub fn get_mut(&mut self, i: EdgeIndex) -> Option<&mut Edge> {
        self.v[i.row][i.col][i.ty].as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;
    use std::time;

    #[test]
    fn test_random_traverse() {
        let g = Graph::new(1000, 1000);

        let mut cur_node = g.get_node([0, 0].into());

        for _ in 0..100000 {
            let mut n = random::<usize>() % 6;
            //println!("at node: {:?}", cur_node.index);
            loop {
                if let Some(v) = cur_node.get_adjacent_node_n(n, &g.edge_matrix) {
                    cur_node = g.node_matrix.get(v);
                    break;
                } else {
                    n += 1;
                    n %= 6;
                }
            }
        }
    }

    #[test]
    fn test_verify_graph() {
        let mut g = Graph::new(1000, 1000);


    }

    #[test]
    fn test_update_edge_stresses() {
        let mut g = Graph::new(1080, 1920);
        
        let t = time::Instant::now();
        g.update_graph_edge_stresses();
        println!("time: {}", t.elapsed().as_micros());
    }
}