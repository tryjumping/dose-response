macro_rules! error {
    ($message:expr) => {
        return core::result::Result::Err(std::boxed::Box::new(crate::error::Error::new($message)))
    };
}
