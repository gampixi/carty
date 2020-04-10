use num_derive::{FromPrimitive, ToPrimitive};

#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum ResponseErrorCode {
    UnknownRequest = 0xBEEF,
    NameAlreadyUsed = 0xE001,
    Unauthorized = 0xE002,
    Irrelevant = 0xE003,
    Illegal = 0xE004,
    MiscError = 0xEFFF,
}