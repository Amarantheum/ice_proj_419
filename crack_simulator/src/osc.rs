use std::{sync::{Arc, Mutex}, collections::VecDeque, error::Error};
use rosc::{encoder, OscType, OscMessage, OscPacket};
use std::net::{SocketAddrV4, Ipv4Addr, UdpSocket};
use std::fs::File;
use std::io::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
struct IpSettings {
    audio_ip: String,
    audio_port: u16,
    watch_ip: String,
    watch_port: u16,
}

fn read_settings() -> Result<IpSettings, Box<dyn Error>> {
    let mut file = File::open("settings.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let settings: IpSettings = serde_json::from_str(&contents)?;
    Ok(settings)
}

pub fn read_watch_task(crack_update_buf: Arc<Mutex<VecDeque<f32>>>, stop: Arc<Mutex<bool>>) {
    // initialize sockets
    const MOVING_AVG_SIZE: usize = 4;
    let settings = read_settings()
        .expect("failed to read settingsg file");
    let audio_target = SocketAddrV4::new(
        Ipv4Addr::from_str(settings.audio_ip.as_str())
            .expect("failed to parse audio_ip as an ipv4 address"),
        settings.audio_port
    );
    let watch_src = SocketAddrV4::new(
        Ipv4Addr::from_str(settings.watch_ip.as_str())
            .expect("failed to parse audio_ip as an ipv4 address"),
        settings.watch_port
    );
    let watch_socket = UdpSocket::bind(watch_src).unwrap();

    let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {                     
        addr: "/start".to_string(),
        args: vec![],
    }))
    .unwrap();
    watch_socket.send_to(&msg_buf, &audio_target).unwrap();

    // spawn task
    std::thread::spawn(move || {
        
        let mut buf = [0u8; rosc::decoder::MTU];
        let mut time = std::time::Instant::now();
        let mut moving_avg_buf = VecDeque::from(vec![0.0; MOVING_AVG_SIZE]);
        loop {
            match watch_socket.recv_from(&mut buf) {
                Ok((size, _addr)) => {
                    if *stop.lock().unwrap() {
                        println!("stopping");
                        let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
                                
                            addr: "/stop".to_string(),
                            args: vec![
                                OscType::Int(0),
                            ],
                        }))
                        .unwrap();
                        watch_socket.send_to(&msg_buf, &audio_target).unwrap();
                        break;
                    }
                    //println!("Received packet with size {} from: {}", size, addr);
                    let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                    if let Ok(v) = handle_packet(packet) {
                        moving_avg_buf.pop_front();
                        moving_avg_buf.push_back(v);
                        let mut moving_avg = 0.0;
                        for i in 0..MOVING_AVG_SIZE {
                            moving_avg += moving_avg_buf[i];
                        }
                        moving_avg /= MOVING_AVG_SIZE as f32;
                        
                        if moving_avg > 500.0  && time.elapsed().as_secs_f32() > 0.5 {
                            crack_update_buf.lock().unwrap().push_back(moving_avg);
                            time = std::time::Instant::now();

                            let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
                                
                                addr: "/crack".to_string(),
                                args: vec![
                                    OscType::Float(moving_avg)
                                ],
                            }))
                            .unwrap();
                            watch_socket.send_to(&msg_buf, &audio_target).unwrap();
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
