use std::sync::{Mutex, Arc};
use std::time::SystemTime;
use crate::client::Client;

#[derive(Debug)]
pub struct Room {
    pub id: u64,
    pub name: String,
    pub max_clients: u16,
    pub clients: Mutex<Vec<Arc<Client>>>,
    pub refresh_time: SystemTime,
}