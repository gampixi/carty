use num_derive::{FromPrimitive, ToPrimitive};

#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum ResponseHeader {
    Good = 0x1111,
    Error = 0xDEAD
}

#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum RequestHeader {
    RegisterPlayer = 0x0001,
    UnregisterPlayer = 0x000F,
    ConfirmPlayerRegister = 0x0002,
    CreateRoom = 0x1001,
    JoinRoom = 0x1011,
    LeaveRoom = 0x101F,
    GetRoomOverview = 0x1100,
    GetRoomPlayers = 0x1101,
    GetRoomGameState = 0x1102,
}