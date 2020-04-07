use std::net::{TcpStream, TcpListener};
use std::io::prelude::*;
use std::io::{Write, BufReader};
use threadpool::ThreadPool;
use std::rc::Rc;
use std::sync::{Mutex, Arc};
use rand::prelude::*;
use std::thread;
use crossbeam::*;

enum ResponseHeader {
    PlayerConnected = 0xAAEE,
    PlayerDisconnected = 0xEEAA,
    Error = 0xDEAD
}

enum ResponseErrorCode {
    UnknownRequest = 0xAAAA,
    NameAlreadyUsed = 0xF00F
}

enum RequestHeader {
    ConnectPlayer = 0xFEFE,
    DisconnectPlayer = 0xEFEF
}

struct CartyClient {
    id: u64,
    secret: u64,
    name: String,
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
}

impl CartyConnection { 
    fn new(address: &str, server: CartyServer) -> CartyConnection {
        let tcp_listener = TcpListener::bind(&address).unwrap();

        CartyConnection {
            tcp: tcp_listener,
            server: Arc::new(server),
        }
    }

    fn handle_requests(&self) {
        crossbeam::scope(|s| {
            for stream in self.tcp.incoming() {
                println!("New request incoming!");
                let stream = stream.unwrap();   
                s.spawn(|_| {
                    self.server.clone().handle_request(stream);
                });
            }
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

    fn add_client(&self, name: &str) -> Arc<Mutex<CartyClient>> {
        let new_client = CartyClient {
            name: String::from(name),
            id: *self.current_user_id.lock().unwrap(),
            secret: rand::random::<u64>(),
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
        new_client.clone()
    }

    fn handle_request(&self, mut stream: TcpStream) {
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut data = String::new();
        let len = reader.read_line(&mut data).unwrap();
        println!("Incoming request: {}", data.trim());
        thread::sleep(std::time::Duration::from_millis(1000));

        let new_client = self.add_client(&data);
        {
            let new_client = new_client.lock().unwrap();

            stream.write(&HEADER_PLAYER_ADD.to_be_bytes());
            stream.write(&new_client.secret.to_be_bytes()).expect("Response failed");
            println!("Request finished: {}", &new_client.id);
        }
        thread::sleep(std::time::Duration::from_millis(1000));
        stream.flush().unwrap();
    }
}

fn main() {
    let server = CartyServer::new();
    let mut connection = CartyConnection::new("127.0.0.1:42069", server);

    connection.handle_requests();
}