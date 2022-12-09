use graphics::SimulationScreen;
use lazy_static::lazy_static;
use osc::read_watch_task;
use std::{sync::{Arc, Mutex, RwLock}, collections::VecDeque};
use std::io;
use std::str::FromStr;
use std::time::Instant;
use rand::random;

use crate::osc::CrackNotifier;

mod simulation;
mod graphics;
mod osc;

lazy_static!{
    static ref REPEAT_AMT: RwLock<usize> = RwLock::new(0);
    static ref TIMER: RwLock<Instant> = RwLock::new(Instant::now());
    static ref NUM_CRACKS: usize = 47;
}

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
        let crack_notifier = Arc::new(Mutex::new(CrackNotifier::new(Arc::clone(&crack_update_buf))
            .expect("failed to create crack_notifier")));
        let crack_notifier_tmp = Arc::clone(&crack_notifier);
        let mut t = (*TIMER).write().unwrap();
        *t = Instant::now();
        drop(t);
        std::thread::spawn(move || {
            read_watch_task(crack_notifier_tmp, Arc::clone(&stop_tmp));
        });
        input = String::new();
        while input.trim() != "stop" {
            input = String::new();
            if let Err(e) = io::stdin().read_line(&mut input) {
                println!("err reading line: {:?}", e);
            }
            if let Ok(f) = f32::from_str(input.trim()) {
                CrackNotifier::send_cloned_crack(&crack_notifier, f);
            } else if input.trim() == "" {
                CrackNotifier::send_cloned_crack(&crack_notifier, random::<f32>() * 500_f32 + 500_f32);
            }
        }
        println!("stopping simulation...");
        *stop.lock().unwrap() = true;
    });

    // run simulation
    simulation.run();
}