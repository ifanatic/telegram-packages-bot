use std::error::Error as StdError;
use std::fmt::{self, Debug, Display, Formatter};
use std::io;
use hyper::Error as HyperError;
use rustc_serialize::json::DecoderError;
use telegram_bot;

pub enum Error {
    General(String),
}

impl From<HyperError> for Error {
    fn from(e: HyperError) -> Error {
        Error::General(e.description().to_owned())
    }
}

impl From<DecoderError> for Error {
    fn from(e: DecoderError) -> Error {
        Error::General(e.description().to_owned())
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::General(e.description().to_owned())
    }
}

impl From<telegram_bot::Error> for Error {
    fn from(e: telegram_bot::Error) -> Error {
        Error::General(e.description().to_owned())
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::General(ref msg) => write!(f, "error: {}", msg).unwrap(),
        };

        Ok(())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::General(ref msg) => write!(f, "error: {}", msg).unwrap(),
        };

        Ok(())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::General(ref msg) => msg,
        }
    }
}