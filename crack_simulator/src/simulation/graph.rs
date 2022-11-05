use super::node::Node;
use super::edge::Edge;

/// Graph of stress nodes
/// Top left corner starts like:
/// * * * *
///  * * * *
/// * * * *
///  * * * * 
struct Graph {
    /// matrix of nodes
    node_matrix: Vec<Vec<Node>>,
    /// matrix of edges. Note there's 3 types of edges at each level
    edge_matrix: Vec<[Vec<Option<Edge>>; 3]>
}

impl Graph {
    /// rows = number of rows (y axis), cols = num calls (x axis)
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut node_matrix = Vec::with_capacity(rows);
        let mut edge_matrix = Vec::with_capacity(rows);

        for _ in 0..rows {
            let mut cur_col = Vec::with_capacity(cols);
            for _ in 0..cols {
                cur_col.push(Node::new(Self::get_init_implicit_node_stress()));
            }
            node_matrix.push(cur_col);
        }

        for y in 0..rows {
            let mut cur_col = [Vec::with_capacity(cols), Vec::with_capacity(cols), Vec::with_capacity(cols)];
            for i in 0..3 {
                let mut cur_col_type = Vec::with_capacity(cols);
                for x in 0..cols {
                    if y % 2 != 1 {
                        if x != cols - 1 {
                            cur_col_type.push(Edge::new(Self::get_init_implicit_edge_stress(), &mut node_matrix[y][x], 0, &mut node_matrix[y][x + 1], 3));
                        }
                    } else {
                        
                    }
                }
            }
           
            edge_matrix.push(cur_col);
        }



        Self {
            node_matrix,
            edge_matrix,
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