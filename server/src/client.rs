#[derive(Debug)]
pub struct Client {
    pub id: u64,
    pub secret: u64,
    pub name: String,
    pub registered: bool,
    pub room: Option<u64>,
}