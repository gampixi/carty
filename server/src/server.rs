use std::net::{TcpStream, UdpSocket, SocketAddr};
use std::sync::{Mutex, Arc, RwLock};
use std::thread;
use rayon::prelude::*;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::prelude::*;
use std::io::{Write, BufReader, Cursor};
use num_traits::{FromPrimitive};
use std::time::SystemTime;

use crate::client::Client;
use crate::room::Room;
use crate::magic::*;
use crate::actions;

pub struct Server {
    current_seq_id: Mutex<u64>,
    pub clients: RwLock<Vec<Arc<RwLock<Client>>>>,
    pub rooms: RwLock<Vec<Arc<RwLock<Room>>>>
}

impl Server {
    pub fn new() -> Server {
        Server {
            current_seq_id: Mutex::new(1),
            clients: RwLock::new(vec![]),
            rooms: RwLock::new(vec![]),
        }
    }

    pub fn find_client(&self, name: &str) -> Option<Arc<RwLock<Client>>> {
        let clients = self.clients.read().unwrap();
        let result = clients.par_iter().find_any(|&c| c.read().unwrap().name == name);
        match result {
            Some(c) => return Some(c.clone()),
            None => return None
        }
    }

    pub fn find_client_secure(&self, id: u64, secret: u64) -> Result<Arc<RwLock<Client>>, ResponseErrorCode> {
        let clients = self.clients.read().unwrap();
        let result = clients.par_iter().find_first(|&c| c.read().unwrap().id == id);
        match result {
            Some(c) => {
                if c.read().unwrap().secret == secret {
                    return Ok(c.clone())
                }
                return Err(ResponseErrorCode::Unauthorized)
            },
            None => return Err(ResponseErrorCode::Irrelevant)
        }
    }

    pub fn add_client(&self, name: &str) -> Result<Arc<RwLock<Client>>, ResponseErrorCode> {
        // Look if a player with this name is already registered
        let search = self.find_client(name);
        match search {
            Some(_c) => return Err(ResponseErrorCode::NameAlreadyUsed),
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
        let new_client = Arc::new(RwLock::new(new_client));
        {
            let mut clients = self.clients.write().unwrap();
            clients.push(new_client.clone());
        }
        Ok(new_client.clone())
    }

    pub fn remove_client(&self, client: Arc<RwLock<Client>>) -> Result<(), ResponseErrorCode> {
        let mut clients = self.clients.write().unwrap();
        let client_id = client.read().unwrap().id;
        match clients.par_iter().position_first(|c| c.read().unwrap().id == client_id) {
            Some(pos) => {
                clients.remove(pos);
            },
            None => {
                return Err(ResponseErrorCode::MiscError);
            }
        }
        return Ok(());
    }

    pub fn find_room(&self, name: &str) -> Option<Arc<RwLock<Room>>> {
        let rooms = self.rooms.read().unwrap();
        let result = rooms.par_iter().find_any(|&c| c.read().unwrap().name == name);
        match result {
            Some(c) => return Some(c.clone()),
            None => return None
        }
    }

    pub fn find_room_id(&self, id: u64) -> Option<Arc<RwLock<Room>>> {
        let rooms = self.rooms.read().unwrap();
        let result = rooms.par_iter().find_any(|&c| c.read().unwrap().id == id);
        match result {
            Some(c) => return Some(c.clone()),
            None => return None
        }
    }

    pub fn add_room(&self, name: &str) -> Result<Arc<RwLock<Room>>, ResponseErrorCode> {
        // Look if a room with this name is already registered
        let search = self.find_room(name);
        match search {
            Some(_c) => return Err(ResponseErrorCode::NameAlreadyUsed),
            None => (),
        }

        let new_room = Room {
            name: String::from(name),
            id: *self.current_seq_id.lock().unwrap(),
            max_clients: 8,
            clients: Mutex::new(vec![]),
            refresh_time: SystemTime::now()
        };
        {
            let mut ids = self.current_seq_id.lock().unwrap();
            *ids += 1;
        }
        let new_room = Arc::new(RwLock::new(new_room));
        {
            let mut rooms = self.rooms.write().unwrap();
            rooms.push(new_room.clone());
        }
        Ok(new_room.clone())
    }

    pub fn remove_room(&self, room: Arc<RwLock<Room>>) -> Result<(), ResponseErrorCode> {
        let mut rooms = self.rooms.write().unwrap();
        let room_id = room.read().unwrap().id;
        match rooms.par_iter().position_first(|c| c.read().unwrap().id == room_id) {
            Some(pos) => {
                rooms.remove(pos);
            },
            None => {
                return Err(ResponseErrorCode::MiscError);
            }
        }
        return Ok(());
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
                actions::register_client(&new_name, &self, &stream)?;
            }
            Some(RequestHeader::UnregisterPlayer) => {
                let mut id = [0; 8];
                let mut secret = [0; 8];
                reader.read_exact(&mut id).unwrap();
                let id = Cursor::new(id).read_u64::<BigEndian>().unwrap();
                reader.read_exact(&mut secret).unwrap();
                let secret = Cursor::new(secret).read_u64::<BigEndian>().unwrap();
                actions::unregister_client(id, secret, &self, &stream)?;
            }
            Some(RequestHeader::ConfirmPlayerRegister) => {
                let mut id = [0; 8];
                let mut secret = [0; 8];
                reader.read_exact(&mut id).unwrap();
                let id = Cursor::new(id).read_u64::<BigEndian>().unwrap();
                reader.read_exact(&mut secret).unwrap();
                let secret = Cursor::new(secret).read_u64::<BigEndian>().unwrap();
                actions::confirm_client_register(id, secret, &self, &stream)?;
            }
            Some(RequestHeader::CreateRoom) => {
                let mut id = [0; 8];
                let mut secret = [0; 8];
                let mut new_name = String::new();
                reader.read_exact(&mut id).unwrap();
                let id = Cursor::new(id).read_u64::<BigEndian>().unwrap();
                reader.read_exact(&mut secret).unwrap();
                let secret = Cursor::new(secret).read_u64::<BigEndian>().unwrap();
                reader.read_line(&mut new_name).unwrap();
                new_name = new_name.trim().to_string();
                actions::create_room(id, secret, &new_name, &self, &stream)?;
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