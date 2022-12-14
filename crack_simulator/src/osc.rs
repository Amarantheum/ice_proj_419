use std::{sync::{Arc, Mutex}, collections::VecDeque, error::Error, time::Duration};
use rosc::{encoder, OscType, OscMessage, OscPacket};
use std::net::{SocketAddrV4, Ipv4Addr, UdpSocket};
use std::fs::File;
use std::io::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use rand::random;
use colored::Colorize;

use crate::REPEAT_AMT;

#[derive(Serialize, Deserialize)]
struct IpSettings {
    audio_ip: String,
    audio_port: u16,
    src_ip: String,
    watch_port: u16,
    src_port: u16,
}

pub struct CrackNotifier {
    audio_target: SocketAddrV4,
    udp_socket: UdpSocket,
    update_buf: Arc<Mutex<VecDeque<f32>>>,
}

fn read_settings() -> Result<IpSettings, Box<dyn Error>> {
    let mut file = File::open("settings.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let settings: IpSettings = serde_json::from_str(&contents)?;
    Ok(settings)
}


impl CrackNotifier {
    #[inline]
    pub fn new(update_buf: Arc<Mutex<VecDeque<f32>>>) -> Result<Self, Box<dyn Error>> {
        let settings = read_settings()?;
        let audio_target = SocketAddrV4::new(
            Ipv4Addr::from_str(settings.audio_ip.as_str())
                .expect("failed to parse audio_ip as an ipv4 address"),
            settings.audio_port
        );

        let crack_src = SocketAddrV4::new(
            Ipv4Addr::from_str(settings.src_ip.as_str())
                .expect("failed to parse audio_ip as an ipv4 address"),
            settings.src_port
        );
        let udp_socket = UdpSocket::bind(crack_src)?;

        Ok(
            Self {
                audio_target,
                udp_socket,
                update_buf,
            }
        )
    }

    #[inline]
    pub fn notify(&self, buf: Option<&Vec<u8>>, crack_value: Option<f32>) {
        if let Some(v) = buf {
            self.udp_socket.send_to(v, &self.audio_target)
                .unwrap();
        }
    
        if let Some(v) = crack_value {
            self.update_buf.lock()
                .unwrap()
                .push_back(v);
        }
    }

    #[inline]
    pub fn send_crack(&self, crack_value: f32) {
        let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
            addr: "/crack".to_string(),
            args: vec![
                OscType::Float(crack_value),
            ],
        }))
        .unwrap();
    
        self.notify(Some(&msg_buf), Some(crack_value));

        
    }

    pub fn send_cloned_crack(notifier: &Arc<Mutex<Self>>, crack_value: f32) {
        let not_ref = notifier.lock().unwrap();
        not_ref.send_crack(crack_value);
        drop(not_ref);

        if *REPEAT_AMT.read().unwrap() > 0 {
            let cloned = Arc::clone(&notifier);
            std::thread::spawn(move || {
                for _ in 0..*REPEAT_AMT.read().unwrap() {
                    std::thread::sleep(Duration::from_millis(random::<u64>() % 500 + 300));
                    let not_ref = cloned.lock().unwrap();
                    not_ref.send_crack((crack_value + (random::<f32>() * 200_f32 - 100_f32)).max(500_f32).min(1000_f32));
                    drop(not_ref);
                }
            });
        }
    }
}


pub fn read_watch_task(notifier: Arc<Mutex<CrackNotifier>>) {
    // initialize sockets
    const MOVING_AVG_SIZE: usize = 4;

    let settings = read_settings()
        .expect("failed to read settings file");
    let watch_src = SocketAddrV4::new(
        Ipv4Addr::from_str(settings.src_ip.as_str())
            .expect("failed to parse audio_ip as an ipv4 address"),
        settings.watch_port
    );

    let watch_socket = UdpSocket::bind(watch_src)
        .expect(format!("failed to bined to socket {:?}", watch_src).as_str());
    
    let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {                     
        addr: "/start".to_string(),
        args: vec![],
    }))
    .unwrap();
    
    let not_ref = notifier.lock().unwrap();
    not_ref.notify(Some(&msg_buf), None);
    drop(not_ref);

    // spawn task
    std::thread::spawn(move || {
        
        let mut buf = [0u8; rosc::decoder::MTU];
        let mut time = std::time::Instant::now();
        let mut moving_avg_buf = VecDeque::from(vec![0.0; MOVING_AVG_SIZE]);
        loop {
            match watch_socket.recv_from(&mut buf) {
                Ok((size, _addr)) => {
                    let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                    if let Ok(v) = handle_packet(packet) {
                        moving_avg_buf.pop_front();
                        moving_avg_buf.push_back(v);
                        let mut moving_avg = 0.0;
                        for i in 0..MOVING_AVG_SIZE {
                            moving_avg += moving_avg_buf[i];
                        }
                        moving_avg /= MOVING_AVG_SIZE as f32;
                        
                        if moving_avg > 500.0  && time.elapsed().as_secs_f32() > 2.0 {
                            time = std::time::Instant::now();
                            CrackNotifier::send_cloned_crack(&notifier, moving_avg);
                        }
                    }
                }
                Err(e) => {
                    println!("Error receiving from socket: {}", e);
                    break;
                }
            }
        }
    });
}

fn handle_packet<'a>(packet: OscPacket) -> Result<f32, &'a str> {
    match packet {
        OscPacket::Message(mut msg) => {
            if msg.addr == "/accel" {
                let mut tmp = 0.0;
                for _ in 0..3 {
                    if let Some(arg) = msg.args.pop() {
                        if let Some(f) = arg.float() {
                            tmp += f * f;
                        } else {
                            return Err("expected array from accel")
                        }
                    } else {
                        return Err("expected arguments from accel")
                    }
                }
                Ok(tmp.sqrt())
            } else {
                Err("expected accel")
            }
        }
        _ => Err("Did not get expected packet type")
    }
}

pub fn time_controller(crack_notifier: Arc<Mutex<CrackNotifier>>) {
    for i in 0..=24 {
        match i {
            0..=10 | 13..=16 | 19..=22 => (),
            11 | 17 | 23 => {
                let string = format!("WARN: {}", i * 10).white().on_magenta().bold();
                println!("{}", string);
            }
            12 => *REPEAT_AMT.write().unwrap() = 1,
            18 => *REPEAT_AMT.write().unwrap() = 2,
            24 => {
                *REPEAT_AMT.write().unwrap() = 4;
                let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {        
                    addr: "/four".to_string(),
                    args: vec![],
                }))
                .unwrap();
                crack_notifier.lock().unwrap().notify(Some(&msg_buf), None)
            },
            _ => unreachable!(),
        }
        let string = format!("TIME: {}", i * 10).green().bold();
        println!("{}", string);
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

pub fn stop(crack_notifier: Arc<Mutex<CrackNotifier>>) {
    println!("sending /stop");
    let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
            
        addr: "/stop".to_string(),
        args: vec![
            OscType::Int(0),
        ],
    }))
    .unwrap();
    crack_notifier.lock().unwrap().notify(Some(&msg_buf), Some(-1_f32));
}