use glium::implement_vertex;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 2],
}

impl From<[f32; 2]> for Vertex {
    fn from(position: [f32; 2]) -> Self {
        Self {
            position,
        }
    }
}

implement_vertex!(Vertex, position);