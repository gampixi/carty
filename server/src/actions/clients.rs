use std::io::prelude::*;
use std::net::{TcpStream, UdpSocket, SocketAddr};
use rayon::prelude::*;
use crate::server::Server;
use crate::magic::{ResponseErrorCode, ResponseHeader};

pub fn register_client(name: &str, server: &Server, mut stream: &TcpStream) -> Result<(), ResponseErrorCode> {
    let new_client = server.add_client(&name)?;
    let new_client_secret: u64;
    let new_client_id: u64;
    {
        let new_client = new_client.read().unwrap();
        new_client_id = new_client.id;
        new_client_secret = new_client.secret;
    }

    stream.write(&(ResponseHeader::Good as u16).to_be_bytes()).expect("Response failed");
    stream.write(&new_client_id.to_be_bytes()).expect("Response failed");
    stream.write(&new_client_secret.to_be_bytes()).expect("Response failed");
    println!("Registered player: {:?}", &new_client.read().unwrap());
    return Ok(());
}

pub fn confirm_client_register(id: u64, secret: u64, server: &Server, mut stream: &TcpStream) -> Result<(), ResponseErrorCode> {
    let this_client = server.find_client_secure(id, secret)?;
    this_client.write().unwrap().registered = true;
    stream.write(&(ResponseHeader::Good as u16).to_be_bytes()).expect("Response failed");
    println!("Player register completed: {:?}", id);
    return Ok(());
}

pub fn unregister_client(id: u64, secret: u64, server: &Server, mut stream: &TcpStream) -> Result<(), ResponseErrorCode> {
    let this_client = server.find_client_secure(id, secret)?;
    println!("Attempting to unregister player: {:?}, player was fully registered: {:?}", id, this_client.read().unwrap().registered);
    server.remove_client(this_client)?;
    stream.write(&(ResponseHeader::Good as u16).to_be_bytes()).expect("Response failed");
    return Ok(());
}
