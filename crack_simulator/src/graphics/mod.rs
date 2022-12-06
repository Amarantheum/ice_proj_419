use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use glium::framebuffer::{RenderBuffer, SimpleFrameBuffer};
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::ElementState;
use glium::glutin::event_loop::{ControlFlow, EventLoopWindowTarget};
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use glium::glutin::window::Fullscreen;
use glium::glutin::{self, event_loop::EventLoop, event::Event};
use glium::texture::SrgbTexture2d;
use glium::{Display, Surface, Program, VertexBuffer};
use glium::uniform;
use vertex::Vertex;
use crate::simulation::graph::edge::EdgeIndex;
use rand::random;

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
    screen_shader_program: Program,
    //bloom_shader_program: Program,
    crack_texture: SrgbTexture2d,
    //bloom_texture: SrgbTexture2d,

    graph: Graph,
    crack_color: [f32; 4],
    crack_update_list: Arc<Mutex<VecDeque<f32>>>,
    count: usize,
}

impl SimulationScreen {
    pub fn new(width: u32, height: u32, crack_update_list: Arc<Mutex<VecDeque<f32>>>) -> Self {
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

        let crack_texture = SrgbTexture2d::empty(&display, width * 4, height * 4)
            .expect("failed to create texture");
        let mut frame_buf = SimpleFrameBuffer::new(&display, &crack_texture)
            .expect("failed to create frame buffer");
        frame_buf.clear_color(0.0, 0.0, 0.0, 0.0);

        // let bloom_texture = SrgbTexture2d::empty(&display, width, height)
        //     .expect("failed to create texture");
        // let mut frame_buf = SimpleFrameBuffer::new(&display, &bloom_texture)
        //     .expect("failed to create frame buffer");
        // frame_buf.clear_color(0.0, 0.0, 0.0, 0.0);

        // initialize screen with black
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        //target.draw(&vertex_buffer, &indices, &crack_shader_program, &uniform! {}, &Default::default()).unwrap();
        target.finish().unwrap();
        
        
        // graph.add_stress(EdgeIndex { row: 500, col: 500, ty: 2 }, 50.0).unwrap();
        // graph.add_stress(EdgeIndex { row: 500, col: 1000, ty: 0 }, 100.0).unwrap();
        // graph.add_stress(EdgeIndex { row: 500, col: 1500, ty: 1 }, 50.0).unwrap();

        let crack_shader_program = Self::init_crack_program(&display);
        let screen_shader_program = Self::init_screen_program(&display);
        let bloom_shader_program = Self::init_bloom_program(&display);
        Self {
            event_loop,
            display,
            width,
            height,

            vertice_update_list: Vec::with_capacity(256),

            crack_shader_program,
            screen_shader_program,
            //bloom_shader_program,

            graph,
            crack_texture,
            //bloom_texture,
            crack_color: [0.5, 1.0, 1.0, 1.0],
            crack_update_list,
            count: 0,
        }
    }

    /// compile the shaders for ice cracks
    fn init_crack_program(display: &Display) -> Program {
        let vertex_shader_src: &str = include_str!("./shaders/crack_vs.glsl");
        let fragment_shader_src: &str = include_str!("./shaders/crack_fs.glsl");
        glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap()
    }

    fn init_screen_program(display: &Display) -> Program {
        let vertex_shader_src: &str = include_str!("./shaders/screen_vs.glsl");
        let fragment_shader_src: &str = include_str!("./shaders/screen_fs.glsl");
        glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap()
    }

    fn init_bloom_program(display: &Display) -> Program {
        let vertex_shader_src: &str = include_str!("./shaders/bloom_vs.glsl");
        let fragment_shader_src: &str = include_str!("./shaders/bloom_fs.glsl");
        glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap()
    }

    fn get_random_edge_index(width: usize, height: usize) -> EdgeIndex {
        EdgeIndex { row: random::<usize>() % width - 2 + 1, col: random::<usize>() % height - 2 + 1, ty: random::<usize>() % 3 }
    }

    const MAX_STRESS: f32 = 10000.0;
    pub fn run(mut self) {
        let mut time = std::time::Instant::now();
        self.event_loop.run(move |ev, _, control_flow| {
            let mut update_list = self.crack_update_list.lock().unwrap();
            while let Some(v) = update_list.pop_front() {
                let pre = (v - 500.0 + 31.0).min(Self::MAX_STRESS);
                let post = (pre / 500.0).powf(2.5_f32) * Self::MAX_STRESS;
                println!("post: {}", post);
                self.graph.add_stress(self.graph.get_random_edge_index(), post).unwrap();

                if self.count < 47 {
                    self.crack_color[0] -= 0.5 / 47_f32;
                    self.crack_color[1] -= 1.0 / 47_f32;
                }
                self.count += 1;
                println!("color: {:?}", self.crack_color);
            }
            drop(update_list);
            if time.elapsed().as_nanos() > 16_666_667 {
                time =std::time::Instant::now();
                let default_vbo: VertexBuffer<Vertex> = glium::VertexBuffer::new(&self.display, &vec![[-1_f32, -1_f32].into(), [1_f32, 1_f32].into(), [-1_f32, 1_f32].into(), [-1_f32, -1_f32].into(), [1_f32, 1_f32].into(), [1_f32, -1_f32].into()]).unwrap();
                let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
                if self.graph.get_update_amt() != 0 {
                    self.graph.update_graph_edge_stresses(Some(&mut self.vertice_update_list));
                    let vertex_buffer = glium::VertexBuffer::new(&self.display, &self.vertice_update_list).unwrap();
                    let mut frame_buf = SimpleFrameBuffer::new(&self.display, &self.crack_texture)
                        .expect("failed to create frame buffer");

                    frame_buf.draw(&vertex_buffer, &indices, &self.crack_shader_program, &uniform! {}, &Default::default())
                        .expect("failed to draw frame");

                    // let mut frame_buf = SimpleFrameBuffer::new(&self.display, &self.bloom_texture)
                    //     .expect("failed to create frame buffer");

                    // frame_buf.draw(&default_vbo, &indices, &self.bloom_shader_program, &uniform! {crack_texture: &self.bloom_texture, crack_color: self.crack_color, bloom_size: 8, scale: 1920_f32}, &Default::default())
                    //     .expect("failed to draw frame");
                }
                
                let mut target = self.display.draw();
                //target.draw(&vertex_buffer, &indices, &self.screen_shader_program, &uniform! {crack_texture: &self.texture, crack_color: self.crack_color}, &Default::default()).unwrap();
                target.draw(&default_vbo, &indices, &self.screen_shader_program, &uniform! {crack_texture: &self.crack_texture, crack_color: self.crack_color}, &Default::default())
                    .expect("failed to draw frame");
                target.finish().unwrap();
                self.vertice_update_list = Vec::with_capacity(256);
                self.graph.update_graph_stress_propagation();
            }
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
