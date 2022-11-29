use std::i128::MIN;
use std::{fs::File, path::Path};
use std::cmp::min;

use node::Node;
use edge::Edge;
use edge_update_list::EdgeUpdateList;
use rand::random;
use rayon::{slice::{ParallelSlice, ParallelSliceMut}, current_num_threads};
use crate::graphics::vertex::Vertex;

use crate::simulation::graph::edge::PROPOGATION_CONST;

use self::{node::NodeIndex, edge::{EdgeUpdateStatus, CRACK_THRESHOLD}};
use self::edge::EdgeIndex;

use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::io::Write;

pub mod node;
pub mod edge;
mod edge_update_list;
mod stress_vec;
mod propagation_vector;

const PROPOGATION_CUTOFF: f32 = CRACK_THRESHOLD / 1000000_f32;
const WEAKEST_PATH_BIAS: f32 = 3_f32;
const MIN_STRESS: f32 = 0.0000001;

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
                let row_height = 
                cur.push(Node::new(r, c));
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
        for e in &mut self.edge_matrix.v {
            for ee in e {
                for eee in ee {
                    if let Some(e) = eee {
                        self.update_edge_list.push(e.index);
                        e.set_scheduled_for_stress_update();
                    }
                }
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
    pub fn get_edge(&self, i: EdgeIndex) -> Option<&Edge> {
        self.edge_matrix.get(i)
    }
    
    #[inline]
    pub fn get_edge_mut(&mut self, i: EdgeIndex) -> Option<&mut Edge> {
        self.edge_matrix.get_mut(i)
    }

    pub fn add_stress(&mut self, i: EdgeIndex, stress: f32) -> Result<(), ()> {
        if let Some(e) = self.edge_matrix.get_mut(i) {
            e.add_stress(stress, &mut self.update_edge_list);
            Ok(())
        } else {
            Err(())
        }
    }

    /// Sets the ndc values for all nodes in the graph
    /// # Arguments
    /// * `screen_width` - The width of the screen in pixels
    /// * `screen_height` - The height of the screen in pixels
    /// * `x_offset` - The offset into the node grid along the x axis measured in col lengths
    /// * `y_offset` - The offset into the node grid along the y axis measured in row lengths
    /// * `x_scale` - The scale of the node grid along the x axis measured in edge lengths
    /// * `y_scale` - The scale of the node grid along the y axis measured in edge lengths
    pub fn set_node_ndcs(&mut self, x_offset: f32, y_offset: f32, x_scale: f32, y_scale: f32) {
        let row_scale = 3_f32.sqrt() / 2_f32 * y_scale;
        for r in 0..self.rows {
            let row_ndc = 1_f32 - 2_f32 * ((r as f32 + y_offset) * 3_f32.sqrt() / 2_f32 * y_scale);
            for c in 0..self.cols {
                let col_ndc = if r % 2 == 0 {
                    (c as f32 - x_offset) * x_scale * 2_f32 - 1_f32
                } else {
                    (c as f32 - x_offset + 0.5) * x_scale * 2_f32 - 1_f32
                };
                self.node_matrix.get_mut([r, c].into()).set_ndc([col_ndc, row_ndc].into());
            }
        }
    }

    fn get_init_implicit_node_stress() -> f32 {
        // TODO randomize here?
        0_f32
    }

    fn get_init_implicit_edge_stress() -> f32 {
        // TODO randomize here?
        random()
    }

    pub fn update_graph_edge_stresses(&mut self, mut triangle_update_list: Option<&mut Vec<Vertex>>) {
        let update_n = self.update_edge_list.size();
        //println!("update edge stress size: {}", update_n);
        for _ in 0..update_n {
            if let Some(e) = self.edge_matrix.get_mut(self.update_edge_list.pop().expect("shouldn't be none")) {
                if e.update_total_stress(&mut self.node_matrix, &mut self.update_edge_list) {
                    // if an edge cracked, add triangles to triangle update list
                    if let Some(l) = triangle_update_list.as_mut() {
                        let a_edges = Edge::get_adjacent_edges(e.index, self.cols);
                        
                        for g in 0..2 {
                            if let Some(oe1) = a_edges[2 * g] {
                                if let Some(oe2) = a_edges[2 * g + 1] {
                                    if let Some(e1) = self.edge_matrix.get(oe1) {
                                        if let Some(e2) = self.edge_matrix.get(oe2) {
                                            let mut ns = [NodeIndex::default(); 3];
                                            ns[0] = e1.nodes[0];
                                            ns[1] = e1.nodes[1];
                                            ns[2] = if e2.nodes[0] == ns[0] || e2.nodes[0] == ns[1] {
                                                e2.nodes[1]
                                            } else {
                                                e2.nodes[0]
                                            };

                                            let test = [self.node_matrix.get(ns[0]).ndc.expect("shouldn't be none"),
                                            self.node_matrix.get(ns[1]).ndc.expect("shouldn't be none"),
                                            self.node_matrix.get(ns[2]).ndc.expect("shouldn't be none")];

                                            //println!("triangle: {:?}", test);
    
                                            for i in 0..3 {
                                                l.push(self.node_matrix.get(ns[i]).ndc.expect("shouldn't be none"));
                                            }
                                            
                                        }
                                    }
                                }
                            }
                        }
                        
                    }
                }
            }
        }
    }

    fn weak_path_bias_fn(ratio: f32) -> f32 {
        0.5 * (WEAKEST_PATH_BIAS * (ratio - 1_f32)).tanh() + 0.5
    }

    pub fn update_graph_stress_propagation(&mut self) {
        let update_n = self.update_edge_list.size();
        //println!("update stress propagation size: {}", update_n);
        for _ in 0..update_n {
            // get next edge
            let e = self.update_edge_list.pop()
                .expect("shouldn't be none");
            
            /*
                // CALCULATION WILL LOOK LIKE
                // state
                imp_stress
                imp_stress_update
                prop_vec
                prop_vec_update

                // adding new prop_vec =>
                prop_vec_update = norm((imp_stress_update * prop_vec_update) + (new_imp_stress * new_prop_vec_update))
                imp_stress_update += new_imp_stress

                // update self
                prop_vec = norm((imp_stress * prop_vec) + (imp_stress_update * prop_vec_update))
                imp_stress += imp_stress_update
                
                // Propagation will look like:
                angle = dot(prop_vec, a.dir)
                a.imp_stress_update = angle * imp_stress
                a.prop_vec_update = prop_vec
            */

            let edge = self.edge_matrix.get(e).unwrap();
            if edge.prop_vec.is_zero() {
                //println!("prop vec ZERO index {:?}", edge.index);
                // add cur stress in both directions
                let mut dir = edge.ty_to_prop_vec();
                let added_stress = edge.stress;

                let adjacent_edges = Edge::get_adjacent_edges(edge.index, self.cols);
                let mut prop_amounts: [Option<f32>; 4] = [None; 4];
                for i in 0..4 {
                    if let Some(e) = adjacent_edges[i] {
                        if let Some(ee) = self.edge_matrix.get(e) {
                            //println!("EDGE {} has stress: {}", i, ee.stress);
                            prop_amounts[i] = Some(ee.stress);
                            // ee.add_stress_update(3_f32.sqrt() / 2.0 * added_stress * PROPOGATION_CONST, dir);
                            // if ee.get_update_status() != EdgeUpdateStatus::StressUpdate {
                            //     ee.set_scheduled_for_stress_update();
                            //     self.update_edge_list.push(ee.index);
                            // }
                        }
                    }
                }

                let mut i = 0;
                for _ in 0..2 {
                    if prop_amounts[i].is_some() && prop_amounts[i + 1].is_some() {
                        let mut scaled_props: [f32; 2] = [0_f32; 2];
                        // unwrapped prop_amts
                        let prop_amounts_ = [prop_amounts[i].unwrap().max(MIN_STRESS), prop_amounts[i + 1].unwrap().max(MIN_STRESS)];
                        scaled_props[0] = Self::weak_path_bias_fn(prop_amounts_[0] / prop_amounts_[1]);
                        scaled_props[1] = 1_f32 - scaled_props[0];

                        //println!("Scaled props {:?}", scaled_props);
                        
                        for j in 0..2 {
                            let e = self.edge_matrix.get_mut(adjacent_edges[i + j].unwrap()).unwrap();
                            let amt = scaled_props[j] * added_stress * PROPOGATION_CONST;
                            if amt > MIN_STRESS {
                                e.add_stress_update(scaled_props[j] * added_stress * PROPOGATION_CONST, dir);
                                if e.get_update_status() != EdgeUpdateStatus::StressUpdate {
                                    e.set_scheduled_for_stress_update();
                                    self.update_edge_list.push(e.index);
                                }
                            }
                            
                        }
                    }
                    dir = -dir;
                    i = 2;
                }
                
                

            } else {
                //println!("prop non zero index: {:?}, dir: {:?}", edge.index, edge.prop_vec);
                // propogate stress in the direction of this pvec
                let dir = edge.prop_vec;
                let added_stress = edge.stress;
                
                let adjacent_edges = Edge::get_adjacent_edges(edge.index, self.cols);
                let inversions = match edge.index.ty {
                    0 => [1_f32, 1_f32, -1_f32, -1_f32],
                    1 => [-1_f32, 1_f32, -1_f32, 1_f32],
                    2 => [1_f32, -1_f32, 1_f32, -1_f32],
                    _ => unreachable!(),
                };

                let mut pre_prop_ratio = [(0_f32, None); 2];
                let mut a = 0;
                for i in 0..4 {
                    if let Some(e) = adjacent_edges[i] {
                        if let Some(ee) = self.edge_matrix.get_mut(e) {
                            let amt = (ee.ty_to_prop_vec() * dir) * inversions[i];
                            if amt > 0_f32 {
                                //println!("added edge {:?} with amt: {}", ee.index, amt);
                                pre_prop_ratio[a] = (amt.max(MIN_STRESS), Some(e));
                                a += 1
                            }
                        }
                    }
                }
                debug_assert!(a < 3);

                if a == 2 {
                    //println!("found 2");
                    let mut scaled_props = [0_f32; 2];
                    scaled_props[0] = Self::weak_path_bias_fn(pre_prop_ratio[0].0 / pre_prop_ratio[1].0);
                    scaled_props[1] = 1_f32 - scaled_props[0];

                    //println!("scaled: {:?}", scaled_props);

                    for i in 0..2 {
                        let e = self.edge_matrix.get_mut(pre_prop_ratio[i].1.unwrap()).unwrap();
                        let amt = scaled_props[i] * added_stress * PROPOGATION_CONST;
                        if amt > MIN_STRESS {
                            e.add_stress_update(scaled_props[i] * added_stress * PROPOGATION_CONST, dir);
                            if e.get_update_status() != EdgeUpdateStatus::StressUpdate {
                                e.set_scheduled_for_stress_update();
                                self.update_edge_list.push(e.index);
                            }
                        }
                        
                    }
                }
            }
            let edge = self.edge_matrix.get_mut(e).unwrap();
            edge.prop_vec = Default::default();
            edge.stress = 0.0;
            edge.set_update_status_propogated();
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
        self.update_graph_edge_stresses(None);
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

    #[test]
    fn test_verify_graph() {
        let g = Graph::new(1000, 1000);
        for r in 0..g.rows {
            for c in 0..g.cols {
                g.node_matrix.v[r][c].verify(&g.node_matrix, &g.edge_matrix, g.rows, g.cols, [r, c].into());
            }
        }
    }

    #[test]
    fn test_update_edge_stresses() {
        let mut g = Graph::new(1080, 1920);

        let t = time::Instant::now();
        g.update_graph_edge_stresses(None);
        println!("time: {}", t.elapsed().as_millis());
        
        let t = time::Instant::now();
        g.update_graph_edge_stresses(None);
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
        let mut g = Graph::new(10, 10);
        g.main_loop();
        // g.edge_matrix.get_mut(EdgeIndex { row: 50, col: 50, ty: 0 }).unwrap().add_stress(100.0, &mut g.update_edge_list);
        // g.edge_matrix.get_mut(EdgeIndex { row: 40, col: 40, ty: 1 }).unwrap().add_stress(100.0, &mut g.update_edge_list);
        // g.edge_matrix.get_mut(EdgeIndex { row: 40, col: 60, ty: 2 }).unwrap().add_stress(100.0, &mut g.update_edge_list);
        
        for _ in 0..100 {
            let t = time::Instant::now();
            g.main_loop();
            println!("main loop time: {}", t.elapsed().as_micros());
        }

        g.debug_print(Some(Path::new("test")));
    }
}