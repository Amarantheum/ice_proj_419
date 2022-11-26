use std::{fs::File, path::Path};

use node::Node;
use edge::Edge;
use edge_update_list::EdgeUpdateList;
use rand::random;
use rayon::{slice::{ParallelSlice, ParallelSliceMut}, current_num_threads};

use self::{node::NodeIndex, edge::EdgeUpdateStatus};
use self::edge::EdgeIndex;

use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::io::Write;

pub mod node;
pub mod edge;
mod edge_update_list;
mod stress_vec;
mod propagation_vector;

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

    update_edge_list: EdgeUpdateList,
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
            update_edge_list: EdgeUpdateList::new(rows * cols * 3),
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
                cur.push(Node::new(Self::get_init_implicit_node_stress(), r, c));
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
        debug_assert!(self.update_edge_list.size() == 0);
        for v in &self.node_matrix.v {
            for vv in v {
                vv.add_to_update_list(&mut self.update_edge_list, &mut self.edge_matrix);
            }
        }
        debug_assert!(self.update_edge_list.size() < self.rows * self.cols * 3);
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
        random()
    }

    fn get_init_implicit_edge_stress() -> f32 {
        // TODO randomize here?
        0_f32
    }

    pub fn update_graph_edge_stresses(&mut self) {
        let update_n = self.update_edge_list.size();
        println!("update edge stress size: {}", update_n);
        for _ in 0..update_n {
            if let Some(e) = self.edge_matrix.get_mut(self.update_edge_list.pop().expect("shouldn't be none")) {
                e.update_total_stress(&mut self.node_matrix, &mut self.update_edge_list);
            }
        }
    }

    pub fn update_graph_stress_propagation(&mut self) {
        let update_n = self.update_edge_list.size();
        println!("update stress propagation size: {}", update_n);
        for _ in 0..update_n {
            // get next edge
            let e = self.update_edge_list.pop()
                .expect("shouldn't be none");
            // set not scheduled to update as long as it isn't scheduled for stress update
            self.edge_matrix.get_mut(e).expect("shouldn't be none").set_update_status_propogated();
            // get orthogonal nodes
            let orth_nodes = self.edge_matrix.get(e)
                .expect("shouldn't be none")
                .get_orthogonal_nodes(&self.node_matrix, &self.edge_matrix);
            // propogate stress
            self.edge_matrix.get_mut(e)
                .expect("shouldn't be none")
                .propogate_stress(&mut self.node_matrix, orth_nodes, &mut self.update_edge_list);
            // add edges to update to update list    
            for n in orth_nodes {
                if let Some(nn) = n {
                    self.node_matrix.get(nn).add_to_update_list(&mut self.update_edge_list, &mut self.edge_matrix);
                }
            }
        }
    }

    fn valid_edge_assert_not(&mut self, s: EdgeUpdateStatus) -> bool {
        for e in &self.update_edge_list.v {
            if self.edge_matrix.get(*e).expect("shouldn't be none").get_update_status() == s {
                return false;
            }
        }
        true
    }

    pub fn main_loop(&mut self) {
        self.update_graph_edge_stresses();
        debug_assert!(self.valid_edge_assert_not(EdgeUpdateStatus::StressUpdate));
        self.update_graph_stress_propagation();
        debug_assert!(self.valid_edge_assert_not(EdgeUpdateStatus::PropogationUpdate));
    }

    /*
    print self in this style:
    o---o---o
     \ / \ / \
      o---o---o
    */
    pub fn debug_print(&self, file: Option<&Path>) {
        let mut output = String::new();
        for r in 0..self.rows {
            let mut line1 = String::new();
            let mut line2 = String::new();
            
            if r % 2 != 0 {
                line1 += "  ";
            } else {
                line2 += " ";
            }
            for c in 0..self.cols {
                line1 += "o";
                let n = self.node_matrix.get([r, c].into());
                if n.edges[0].is_some() && !self.edge_matrix.get(n.edges[0].unwrap()).unwrap().cracked {
                    line1 += "---";
                } else {
                    line1 += "   ";
                }
                if c != 0 || r % 2 != 0 {
                    if n.edges[4].is_some() && !self.edge_matrix.get(n.edges[4].unwrap()).unwrap().cracked {
                        line2 += " / ";
                    } else {
                        line2 += "   ";
                    }
                }
                
                if n.edges[5].is_some() && !self.edge_matrix.get(n.edges[5].unwrap()).unwrap().cracked {
                    line2 += "\\";
                } else {
                    line2 += " ";
                }
            }
            line1 += "\n";
            line2 += "\n";

            output += line1.as_str();
            output += line2.as_str();
        }
        if let Some(p) = file {
            let mut f = File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(p)
                .expect("failed to open file");
            write!(f, "{}", output).unwrap();
        } else {
            println!("{}", output);
        }
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

    //#[test]
    fn test_verify_graph() {
        let g = Graph::new(1000, 1000);
        for r in 0..g.rows {
            for c in 0..g.cols {
                g.node_matrix.v[r][c].verify(&g.node_matrix, &g.edge_matrix, g.rows, g.cols, [r, c].into());
            }
        }
    }

    //#[test]
    fn test_update_edge_stresses() {
        let mut g = Graph::new(1080, 1920);

        let t = time::Instant::now();
        g.update_graph_edge_stresses();
        println!("time: {}", t.elapsed().as_millis());
        
        let t = time::Instant::now();
        g.update_graph_edge_stresses();
        println!("time: {}", t.elapsed().as_nanos());
    }

    #[test]
    fn test_main_loop() {
        println!("main loop");
        let mut g = Graph::new(1080, 1920);
        assert!(g.update_edge_list.size() <= 1080 * 1920 * 3);
        for _ in 0..10 {
            let t = time::Instant::now();
            g.main_loop();
            println!("main loop time: {}", t.elapsed().as_nanos());
        }
    }

    #[test]
    fn test_debug_print() {
        println!("main loop");
        let mut g = Graph::new(100, 100);
        for _ in 0..10 {
            let t = time::Instant::now();
            g.main_loop();
            println!("main loop time: {}", t.elapsed().as_nanos());
        }

        g.debug_print(Some(Path::new("test")));
    }
}