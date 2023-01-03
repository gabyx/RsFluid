use std::error;

pub use simple_error::SimpleResult as SimpleResult;
pub use simple_error::bail as bail;

pub type GenericResult<T> = std::result::Result<T, Box<dyn error::Error>>;

