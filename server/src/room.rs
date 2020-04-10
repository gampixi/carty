use std::sync::{Mutex, Arc};
use crate::client::Client;

pub struct Room {
    id: u64,
    name: String,
    max_clients: u16,
    clients: Mutex<Vec<Arc<Client>>>,
}