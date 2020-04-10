use rayon::prelude::*;
use std::sync::{Mutex, Arc};
use std::time::{Duration, SystemTime};
use crate::server::Server;
use crate::room::Room;

pub fn room_maintenance(server: &Server) {
    let rooms_to_purge: Mutex<Vec<u64>> = Mutex::new(vec![]);
    {
        let rooms = server.rooms.read().unwrap();
        rooms.par_iter().for_each(|r| {
            let mut room = r.write().unwrap();
            if room.clients.lock().unwrap().len() > 0 {
                room.refresh_time = SystemTime::now();
            }
            else {
                match room.refresh_time.elapsed() {
                    Ok(len) => {
                        if len > Duration::from_secs(10) {
                            rooms_to_purge.lock().unwrap().push(room.id);
                            println!("Marking room {} for removal", room.name)
                        }
                    }
                    Err(_) => {
                        // Ignore if shenanigans
                    }
                }
            }
        });
    }

    {
        let purge = rooms_to_purge.lock().unwrap();
        if(purge.len() > 0) {
            println!("Cleaning up {} rooms...", purge.len());
            for i in 0..purge.len() {
                match server.find_room_id(purge[i]) {
                    Some(r) => {
                        server.remove_room(r).ok().unwrap();
                    },
                    None => println!("Error happened when deleting the room :(")
                }
            }
            println!("Done cleaning up rooms!");
        }
    }
}