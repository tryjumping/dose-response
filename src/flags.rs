use std::collections::enum_set::CLike;


#[deriving(FromPrimitive)]
pub enum Flags {
    TODO,
}

impl CLike for Flags {
    fn to_uint(&self) -> uint {
        *self as uint
    }

    fn from_uint(value: uint) -> Flags {
        FromPrimitive::from_uint(value).unwrap()
    }
}
