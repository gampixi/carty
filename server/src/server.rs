use std::net::{TcpStream, UdpSocket, SocketAddr};
use std::sync::{Mutex, Arc};
use std::thread;
use rayon::prelude::*;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::prelude::*;
use std::io::{Write, BufReader, Cursor};
use num_traits::{FromPrimitive};

use crate::client::Client;
use crate::room::Room;
use crate::magic::*;

pub struct Server {
    current_seq_id: Mutex<u64>,
    clients: Mutex<Vec<Arc<Mutex<Client>>>>,
    rooms: Mutex<Vec<Room>>
}

impl Server {
    pub fn new() -> Server {
        Server {
            current_seq_id: Mutex::new(1),
            clients: Mutex::new(vec![]),
            rooms: Mutex::new(vec![]),
        }
    }

    pub fn find_client(&self, name: &str) -> Option<Arc<Mutex<Client>>> {
        let clients = self.clients.lock().unwrap();
        let result = clients.par_iter().find_any(|&c| c.lock().unwrap().name == name);
        match result {
            Some(c) => return Some(c.clone()),
            None => return None
        }
    }

    pub fn find_client_secure(&self, id: u64, secret: u64) -> Result<Arc<Mutex<Client>>, ResponseErrorCode> {
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

    pub fn add_client(&self, name: &str) -> Result<Arc<Mutex<Client>>, ResponseErrorCode> {
        // Look if a player with this name is already registered
        let search = self.find_client(name);
        match search {
            Some(c) => return Err(ResponseErrorCode::NameAlreadyUsed),
            None => (),
        }

        let new_client = Client {
            name: String::from(name),
            id: *self.current_seq_id.lock().unwrap(),
            secret: rand::random::<u64>(),
            registered: false,
            room: None
        };
        {
            let mut user_ids = self.current_seq_id.lock().unwrap();
            *user_ids += 1;
        }
        let new_client = Arc::new(Mutex::new(new_client));
        {
            let mut clients = self.clients.lock().unwrap();
            clients.push(new_client.clone());
        }
        Ok(new_client.clone())
    }

    pub fn handle_udp(&self, data: &[u8], socket: Arc<Mutex<UdpSocket>>, origin: &SocketAddr) -> Result<(), ResponseErrorCode> {
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
        Ok(())
    }

    pub fn handle_request(&self, mut stream: &TcpStream) -> Result<(), ResponseErrorCode> {
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut header= [0; 2];
        reader.read_exact(&mut header).unwrap();
        let header = Cursor::new(header).read_u16::<BigEndian>().unwrap();

        match FromPrimitive::from_u16(header) {
            Some(RequestHeader::RegisterPlayer) => {
                let mut new_name = String::new();
                reader.read_line(&mut new_name).unwrap();
                new_name = new_name.trim().to_string();
                let new_client = self.add_client(&new_name)?;
                let new_client_secret: u64;
                let new_client_id: u64;
                {
                    let new_client = new_client.lock().unwrap();
                    new_client_id = new_client.id;
                    new_client_secret = new_client.secret;
                }

                stream.write(&(ResponseHeader::Good as u16).to_be_bytes()).expect("Response failed");
                stream.write(&new_client_id.to_be_bytes()).expect("Response failed");
                stream.write(&new_client_secret.to_be_bytes()).expect("Response failed");
                println!("Registered player: {:?}", &new_client.lock().unwrap());
            }
            Some(RequestHeader::UnregisterPlayer) => {
                let mut id = [0; 8];
                let mut secret = [0; 8];
                reader.read_exact(&mut id).unwrap();
                let id = Cursor::new(id).read_u64::<BigEndian>().unwrap();
                reader.read_exact(&mut secret).unwrap();
                let secret = Cursor::new(secret).read_u64::<BigEndian>().unwrap();
                let this_client = self.find_client_secure(id, secret)?;
                {
                    let mut clients = self.clients.lock().unwrap();
                    let client_id = this_client.lock().unwrap().id;
                    let pos = clients.par_iter().position_first(|c| c.lock().unwrap().id == client_id).unwrap();
                    clients.remove(pos);
                }
                stream.write(&(ResponseHeader::Good as u16).to_be_bytes()).expect("Response failed");
                println!("Unregistered player: {:?}, player was fully registered: {:?}", id, this_client.lock().unwrap().registered);
            }
            Some(RequestHeader::ConfirmPlayerRegister) => {
                let mut id = [0; 8];
                let mut secret = [0; 8];
                reader.read_exact(&mut id).unwrap();
                let id = Cursor::new(id).read_u64::<BigEndian>().unwrap();
                reader.read_exact(&mut secret).unwrap();
                let secret = Cursor::new(secret).read_u64::<BigEndian>().unwrap();
                let this_client = self.find_client_secure(id, secret)?;
                this_client.lock().unwrap().registered = true;
                stream.write(&(ResponseHeader::Good as u16).to_be_bytes()).expect("Response failed");
                println!("Player register completed: {:?}", id);
            }
            Some(_) => {
                return Err(ResponseErrorCode::UnknownRequest);
            }
            None => {
                return Err(ResponseErrorCode::UnknownRequest);
            }
        }

        stream.flush().unwrap();
        return Ok(());
    }
}