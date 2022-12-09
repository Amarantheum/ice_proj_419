use graphics::SimulationScreen;
use osc::read_watch_task;
use std::{sync::{Arc, Mutex}, collections::VecDeque};
use std::io;
use std::str::FromStr;

mod simulation;
mod graphics;
mod osc;

fn main() {
    // initialize simulation and message passing
    let crack_update_buf: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::with_capacity(20)));
    let simulation = SimulationScreen::new(1920, 1080, Arc::clone(&crack_update_buf));
    
    // spawn io handler
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
        let crack_update_buf_clone = Arc::clone(&crack_update_buf);
        std::thread::spawn(move || {
            read_watch_task(crack_update_buf_clone, Arc::clone(&stop_tmp));
        });
        input = String::new();
        while input.trim() != "stop" {
            input = String::new();
            if let Err(e) = io::stdin().read_line(&mut input) {
                println!("err reading line: {:?}", e);
            }
            if let Ok(f) = f32::from_str(input.trim()) {
                crack_update_buf.lock()
                    .unwrap()
                    .push_back(f);
            }
        }
        println!("stopping simulation...");
        *stop.lock().unwrap() = true;
    });

    // run simulation
    simulation.run();
}