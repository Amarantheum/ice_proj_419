use simulation::graph::Graph;
use graphics::SimulationScreen;
use std::{sync::{Arc, Mutex}, collections::VecDeque};
use rosc::{encoder, OscType, OscMessage, OscPacket};
use std::net::{SocketAddrV4, Ipv4Addr, UdpSocket};

mod simulation;
mod graphics;

fn main() {
    let crack_update_buf: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::with_capacity(20)));
    read_watch_task(Arc::clone(&crack_update_buf));
    let simulation = SimulationScreen::new(1920, 1080, crack_update_buf);
    simulation.run();
}

fn read_watch_task(crack_update_buf: Arc<Mutex<VecDeque<f32>>>) {
    const moving_avg_size: usize = 4;
    std::thread::spawn(move || {
        let src = UdpSocket::bind("192.168.137.1:8080").unwrap();
        let target = SocketAddrV4::new(Ipv4Addr::new(192, 168, 137, 46), 10_000);

        let socket = UdpSocket::bind("192.168.137.1:12004").unwrap();
        let mut buf = [0u8; rosc::decoder::MTU];
        let mut time = std::time::Instant::now();
        let mut moving_avg_buf = VecDeque::from(vec![0.0; moving_avg_size]);
        loop {
            match socket.recv_from(&mut buf) {
                Ok((size, _addr)) => {
                    //println!("Received packet with size {} from: {}", size, addr);
                    let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                    if let Ok(v) = handle_packet(packet) {
                        moving_avg_buf.pop_front();
                        moving_avg_buf.push_back(v);
                        let mut moving_avg = 0.0;
                        for i in 0..moving_avg_size {
                            moving_avg += moving_avg_buf[i];
                        }
                        moving_avg /= moving_avg_size as f32;
                        
                        if moving_avg > 500.0  && time.elapsed().as_secs() > 1 {
                            crack_update_buf.lock().unwrap().push_back(moving_avg);
                            time = std::time::Instant::now();

                            let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
                                
                                addr: "/crack".to_string(),
                                args: vec![
                                    OscType::Float(moving_avg)
                                ],
                            }))
                            .unwrap();
                            socket.send_to(&msg_buf, &target).unwrap();
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
