#![allow(dead_code)]

use std::fmt;

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new(message: &str) -> Self {
        Error {
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

macro_rules! error {
    ($message:expr) => {
        return core::result::Result::Err(std::boxed::Box::new(crate::error::Error::new($message)));
    };
}
