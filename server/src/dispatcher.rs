use std::net::{TcpListener, UdpSocket};
use std::sync::{Mutex, Arc};
use std::thread;
use std::io::{ErrorKind};

use crate::server::Server;

pub struct Dispatcher {
    server: Arc<Server>,
    tcp: TcpListener,
    udp: Arc<Mutex<UdpSocket>>,
}

impl Dispatcher { 
    pub fn new(address: &str, server: Server) -> Dispatcher {
        let tcp_listener = TcpListener::bind(&address).unwrap();
        let udp_socket = UdpSocket::bind(&address).unwrap();
        udp_socket.set_nonblocking(true).unwrap();

        Dispatcher {
            tcp: tcp_listener,
            udp: Arc::new(Mutex::new(udp_socket)),
            server: Arc::new(server),
        }
    }

    pub fn handle_requests(&self) {
        crossbeam::scope(|s| {
            s.spawn(|t| {
                for stream in self.tcp.incoming() {
                    let stream = stream.unwrap();   
                    t.spawn(move |_| {
                        match self.server.clone().handle_request(&stream) {
                            Ok(_) => {
                                // All is good :)
                            },
                            Err(err) => {
                                eprintln!("An error occured while processing request: {:?}", err);
                            }
                        }
                    });
                }
            });

            s.spawn(|u| {
                let sleep_duration = std::time::Duration::from_micros(1);
                loop {
                    let mut buf = [0; 512];
                    let result = self.udp.lock().unwrap().recv_from(&mut buf);
                    match result {
                        Ok(res) => {
                            let (_num_bytes, origin) = res;
                            println!("Got shit :thinking:");
                            u.spawn(move |_| {
                                match self.server.handle_udp(&buf, self.udp.clone(), &origin) {
                                    Ok(_) => {
                                        // All is good :)
                                    },
                                    Err(err) => {
                                        eprintln!("An error occured while processing request: {:?}", err);
                                    }
                                }
                            });
                        },
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                            thread::sleep(sleep_duration);
                        }
                        Err(e) => panic!("UDP IO error: {}", e),
                    }
                }
            });
        }).unwrap();
    }
}