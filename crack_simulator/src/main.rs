use graphics::SimulationScreen;
use osc::read_watch_task;
use std::{sync::{Arc, Mutex}, collections::VecDeque};
use std::io;

mod simulation;
mod graphics;
mod osc;

fn main() {
    let crack_update_buf: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::with_capacity(20)));
    let simulation = SimulationScreen::new(1920, 1080, Arc::clone(&crack_update_buf));
    
    std::thread::spawn(move || {
        let mut input = String::new();
        while input.trim() != "start" {
            input = String::new();
            if let Err(e) = io::stdin().read_line(&mut input) {
                println!("err reading line: {:?}", e);
            }
        }

        let stop: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
        let stop_tmp = Arc::clone(&stop);
        std::thread::spawn(move || {
            read_watch_task(crack_update_buf, Arc::clone(&stop_tmp));
        });
        input = String::new();
        while input.trim() != "stop" {
            input = String::new();
            if let Err(e) = io::stdin().read_line(&mut input) {
                println!("err reading line: {:?}", e);
            }
        }
        *stop.lock().unwrap() = true;
    });
    simulation.run();
}