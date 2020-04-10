use std::io::prelude::*;
use std::net::{TcpStream, UdpSocket, SocketAddr};
use rayon::prelude::*;
use crate::server::Server;
use crate::magic::{ResponseErrorCode, ResponseHeader};

pub fn create_room(id: u64, secret: u64, name: &str, server: &Server, mut stream: &TcpStream) -> Result<(), ResponseErrorCode> {
    server.find_client_secure(id, secret)?;
    let new_room = server.add_room(&name)?;
    let new_room_id: u64;
    {
        let new_room = new_room.read().unwrap();
        new_room_id = new_room.id;
    }

    stream.write(&(ResponseHeader::Good as u16).to_be_bytes()).expect("Response failed");
    stream.write(&new_room_id.to_be_bytes()).expect("Response failed");
    println!("Created room: {:?}", &new_room.read().unwrap());
    return Ok(());
}