use std::net::{TcpStream, TcpListener, UdpSocket, SocketAddr};
use std::io::prelude::*;
use std::io::{Write, BufReader, Cursor, ErrorKind};
use std::rc::Rc;
use std::sync::{Mutex, Arc};
use rand::prelude::*;
use std::thread;
use crossbeam::*;
use byteorder::{BigEndian, ReadBytesExt};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use rayon::prelude::*;

#[derive(FromPrimitive, ToPrimitive, Debug)]
enum ResponseHeader {
    PlayerRegistered = 0xAAEE,
    PlayerUnregistered = 0xEEAA,
    Error = 0xDEAD
}

#[derive(FromPrimitive, ToPrimitive, Debug)]
enum ResponseErrorCode {
    UnknownRequest = 0xAAAA,
    NameAlreadyUsed = 0xF00F,
    Unauthorized = 0xFACC,
    Irrelevant = 0x0101
}

#[derive(FromPrimitive, ToPrimitive, Debug)]
enum RequestHeader {
    RegisterPlayer = 0xFEFE,
    UnregisterPlayer = 0xEFEF
}

#[derive(Debug)]
struct CartyClient {
    id: u64,
    secret: u64,
    name: String,
    registered: bool,
}

struct CartyRoom {
    clients: Vec<Arc<CartyClient>>,
}

struct CartyServer {
    current_user_id: Mutex<u64>,
    clients: Mutex<Vec<Arc<Mutex<CartyClient>>>>,
    rooms: Mutex<Vec<CartyRoom>>
}

struct CartyConnection {
    server: Arc<CartyServer>,
    tcp: TcpListener,
    udp: Arc<Mutex<UdpSocket>>,
}

impl CartyConnection { 
    fn new(address: &str, server: CartyServer) -> CartyConnection {
        let tcp_listener = TcpListener::bind(&address).unwrap();
        let udp_socket = UdpSocket::bind(&address).unwrap();
        udp_socket.set_nonblocking(true).unwrap();

        CartyConnection {
            tcp: tcp_listener,
            udp: Arc::new(Mutex::new(udp_socket)),
            server: Arc::new(server),
        }
    }

    fn handle_requests(&self) {
        crossbeam::scope(|s| {
            s.spawn(|t| {
                for stream in self.tcp.incoming() {
                    let stream = stream.unwrap();   
                    t.spawn(|_| {
                        self.server.clone().handle_request(stream);
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
                            let (num_bytes, origin) = res;
                            println!("Got shit :thinking:");
                            u.spawn(move |_| {
                                self.server.handle_udp(&buf, self.udp.clone(), &origin);
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

impl CartyServer {
    fn new() -> CartyServer {
        CartyServer {
            current_user_id: Mutex::new(1),
            clients: Mutex::new(vec![]),
            rooms: Mutex::new(vec![]),
        }
    }

    fn find_client(&self, name: &str) -> Option<Arc<Mutex<CartyClient>>> {
        let clients = self.clients.lock().unwrap();
        let result = clients.par_iter().find_any(|&c| c.lock().unwrap().name == name);
        match result {
            Some(c) => return Some(c.clone()),
            None => return None
        }
    }

    fn find_client_secure(&self, id: u64, secret: u64) -> Result<Arc<Mutex<CartyClient>>, ResponseErrorCode> {
        let clients = self.clients.lock().unwrap();
        let result = clients.par_iter().find_first(|&c| c.lock().unwrap().id == id);
        match result {
            Some(c) => {
                if c.lock().unwrap().secret == secret {
                    return Ok(c.clone())
                }
                return Err(ResponseErrorCode::Unauthorized)
            },
            None => return Err(ResponseErrorCode::Irrelevant)
        }
    }

    fn add_client(&self, name: &str) -> Result<Arc<Mutex<CartyClient>>, ResponseErrorCode> {
        // Look if a player with this name is already registered
        let search = self.find_client(name);
        match search {
            Some(c) => return Err(ResponseErrorCode::NameAlreadyUsed),
            None => (),
        }

        let new_client = CartyClient {
            name: String::from(name),
            id: *self.current_user_id.lock().unwrap(),
            secret: rand::random::<u64>(),
            registered: true
        };
        {
            let mut user_ids = self.current_user_id.lock().unwrap();
            *user_ids += 1;
        }
        let new_client = Arc::new(Mutex::new(new_client));
        {
            let mut clients = self.clients.lock().unwrap();
            clients.push(new_client.clone());
        }
        Ok(new_client.clone())
    }

    fn handle_udp(&self, data: &[u8], socket: Arc<Mutex<UdpSocket>>, origin: &SocketAddr) {
        let mut reader = BufReader::new(data);
        let mut header= [0; 2];
        reader.read_exact(&mut header).unwrap();
        let header = Cursor::new(header).read_u16::<BigEndian>().unwrap();
        println!("Received some junk on UDP, lets sleep for a while now");
        thread::sleep(std::time::Duration::from_secs(1));
        
        match FromPrimitive::from_u16(header) {
            Some(RequestHeader::RegisterPlayer) => {
                println!("It was RegisterPlayer!");
            },
            _ => {
                println!("It was some other shit");
            }
        }

        let mut new_name = String::new();
        reader.read_line(&mut new_name).unwrap();
        new_name = new_name.trim().to_string();

        println!("The rest of data: {}", new_name);
    }

    fn handle_request(&self, mut stream: TcpStream) {
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut header= [0; 2];
        reader.read_exact(&mut header).unwrap();
        let header = Cursor::new(header).read_u16::<BigEndian>().unwrap();

        match FromPrimitive::from_u16(header) {
            Some(RequestHeader::RegisterPlayer) => {
                let mut new_name = String::new();
                reader.read_line(&mut new_name).unwrap();
                new_name = new_name.trim().to_string();
                let new_client = self.add_client(&new_name);
                match new_client {
                    Ok(client) => {
                        let new_client_secret: u64;
                        {
                            let new_client = client.lock().unwrap();
                            new_client_secret = new_client.secret;
                        }

                        stream.write(&(ResponseHeader::PlayerRegistered as u16).to_be_bytes()).expect("Response failed");
                        stream.write(&new_client_secret.to_be_bytes()).expect("Response failed");
                        println!("Registered player: {:?}", &client.lock().unwrap());
                    },
                    Err(error) => {
                        println!("Error occurred during player register: {:?}", &error);
                        stream.write(&(ResponseHeader::Error as u16).to_be_bytes()).expect("Response failed");
                        stream.write(&(error as u16).to_be_bytes()).expect("Response failed");
                    }
                }
            }
            Some(RequestHeader::UnregisterPlayer) => {
                let mut id = [0; 8];
                let mut secret = [0; 8];
                reader.read_exact(&mut id).unwrap();
                let id = Cursor::new(id).read_u64::<BigEndian>().unwrap();
                reader.read_exact(&mut secret).unwrap();
                let secret = Cursor::new(secret).read_u64::<BigEndian>().unwrap();
                let this_client = self.find_client_secure(id, secret);
                println!("Attempting player unregister: ID: {}, secret: {}", id, secret);
                match this_client {
                    Ok(client) => {
                        {
                            let mut clients = self.clients.lock().unwrap();
                            let client_id = client.lock().unwrap().id;
                            let pos = clients.par_iter().position_first(|c| c.lock().unwrap().id == client_id).unwrap();
                            clients.remove(pos);
                        }
                        stream.write(&(ResponseHeader::PlayerUnregistered as u16).to_be_bytes()).expect("Response failed");
                        println!("Unregistered player: {:?}", id);
                    },
                    Err(error) => {
                        println!("Error occurred during player unregister: {:?}", &error);
                        stream.write(&(ResponseHeader::Error as u16).to_be_bytes()).expect("Response failed");
                        stream.write(&(error as u16).to_be_bytes()).expect("Response failed");
                    }
                }
            }
            None => {
                println!("Unknown request received: {:?}", header);
                stream.write(&(ResponseHeader::Error as u16).to_be_bytes()).expect("Response failed");
                stream.write(&(ResponseErrorCode::UnknownRequest as u16).to_be_bytes()).expect("Response failed");
            }
        }

        stream.flush().unwrap();
    }
}

fn main() {
    let server = CartyServer::new();
    let mut connection = CartyConnection::new("127.0.0.1:42069", server);

    connection.handle_requests();
}