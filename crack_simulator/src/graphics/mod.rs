use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::ElementState;
use glium::glutin::event_loop::{ControlFlow, EventLoopWindowTarget};
use glium::glutin::window::Fullscreen;
use glium::glutin::{self, event_loop::EventLoop, event::Event};
use glium::{Display, Surface, Program};
use glium::uniform;
use vertex::Vertex;
use crate::simulation::graph::edge::EdgeIndex;

use crate::simulation::graph::Graph;

mod polygon;
pub mod vertex;

pub struct SimulationScreen {
    pub event_loop: EventLoop<()>,
    pub display: Display,
    pub width: u32,
    pub height: u32,

    vertice_update_list: Vec<Vertex>,
    
    crack_shader_program: Program,

    graph: Graph,
}

impl SimulationScreen {
    pub fn new(width: u32, height: u32) -> Self {
        let t = std::time::Instant::now();
        println!("Building graph...");
        let mut graph = Graph::new((height as f32 / 3_f32.sqrt() * 2_f32).ceil() as usize, width as usize + 1);
        graph.set_node_ndcs(0.5, 0.0, 1.0 / width as f32, (height as f32 / 3_f32.sqrt() * 2_f32).ceil() / (height as f32 / 3_f32.sqrt() * 2_f32) / height as f32);
        graph.update_graph_edge_stresses(None);
        println!("Finished building graph in {} seconds", t.elapsed().as_secs_f32());

        let event_loop = glutin::event_loop::EventLoop::new();
        let wb = glutin::window::WindowBuilder::new()
            .with_inner_size(LogicalSize::new(width, height));
        let cb = glutin::ContextBuilder::new()
            .with_multisampling(8);
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();
        
        let crack_shader_program = Self::init_crack_program(&display);

        /*let vertex1 = Vertex { position: [-0.5, -0.5] };
        let vertex2 = Vertex { position: [ 0.0,  0.5] };
        let vertex3 = Vertex { position: [ 0.5, -0.25] };
        let shape = vec![vertex1, vertex2, vertex3];
        let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);*/

        // initialize screen with black
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        //target.draw(&vertex_buffer, &indices, &crack_shader_program, &uniform! {}, &Default::default()).unwrap();
        target.finish().unwrap();
        //graph.add_stress(EdgeIndex { row: 500, col: 500, ty: 2 }, 100.0).unwrap();
        graph.add_stress(EdgeIndex { row: 500, col: 1000, ty: 0 }, 100.0).unwrap();
        //graph.add_stress(EdgeIndex { row: 500, col: 1500, ty: 1 }, 100.0).unwrap();

        Self {
            event_loop,
            display,
            width,
            height,

            vertice_update_list: Vec::with_capacity(256),

            crack_shader_program,

            graph,
        }
    }

    /// compile the shaders for ice cracks
    fn init_crack_program(display: &Display) -> Program {
        let vertex_shader_src: &str = include_str!("./shaders/crack_vs.glsl");
        let fragment_shader_src: &str = include_str!("./shaders/crack_fs.glsl");
        glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap()
    }

    pub fn run(mut self) {
        self.event_loop.run(move |ev, _, control_flow| {
            self.graph.update_graph_edge_stresses(Some(&mut self.vertice_update_list));
            let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
            let vertex_buffer = glium::VertexBuffer::new(&self.display, &self.vertice_update_list).unwrap();
            let mut target = self.display.draw();
            target.draw(&vertex_buffer, &indices, &self.crack_shader_program, &uniform! {}, &Default::default()).unwrap();
            target.finish().unwrap();
            self.vertice_update_list = Vec::with_capacity(256);
            self.graph.update_graph_stress_propagation();


            let next_frame_time = std::time::Instant::now() +
                std::time::Duration::from_nanos(16_666_667);
            *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
            match ev {
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                        return;
                    },
                    glutin::event::WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(kc) = input.virtual_keycode {
                            use glutin::event::VirtualKeyCode;
                            match kc {
                                // handle esc
                                VirtualKeyCode::Escape => {
                                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                                    return;
                                },
                                // handle fullscreen
                                VirtualKeyCode::F11 => {
                                    if input.state == ElementState::Pressed {
                                        let window = self.display.gl_window();
                                        let window = window.window();
                                        if window.fullscreen().is_some() {
                                            self.display.gl_window().window().set_fullscreen(None)
                                        } else {
                                            self.display.gl_window().window().set_fullscreen(Some(Fullscreen::Borderless(None)))
                                        }
                                    }
                                }
                                _ => (),
                            }   
                        }
                    }
                    _ => return,
                },
                _ => (),
            }
        });
    }
}
