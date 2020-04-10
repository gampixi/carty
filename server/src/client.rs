use crate::magic::*;

#[derive(Debug)]
pub struct Client {
    pub id: u64,
    pub secret: u64,
    pub name: String,
    pub registered: bool,
    pub room: Option<u64>,
}

impl Client {
    pub fn move_to_room(&mut self, room_id: u64) -> Result<(), ResponseErrorCode> {
        Ok(())
    }

    pub fn leave_room(&mut self, room_id: u64) -> Result<(), ResponseErrorCode> {
        Ok(())
    }
}