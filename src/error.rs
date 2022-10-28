use std::fmt::Formatter;
use std::time::SystemTimeError;
use std::{fmt, io, result, time};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

impl Error {
    pub fn new(kind: ErrorKind) -> Error {
        Error(Box::new(kind))
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }

    pub fn into_kind(self) -> ErrorKind {
        *self.0
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Io(io::Error),
    OpenSslStack(openssl::error::ErrorStack),
    SystemTime(time::SystemTimeError),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::new(ErrorKind::Io(err))
    }
}

impl From<openssl::error::ErrorStack> for Error {
    fn from(err: openssl::error::ErrorStack) -> Error {
        Error::new(ErrorKind::OpenSslStack(err))
    }
}

impl From<time::SystemTimeError> for Error {
    fn from(err: SystemTimeError) -> Error {
        Error::new(ErrorKind::SystemTime(err))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self.0 {
            ErrorKind::Io(ref err) => err.fmt(f),
            ErrorKind::OpenSslStack(ref err) => err.fmt(f),
            ErrorKind::SystemTime(ref err) => err.fmt(f),
        }
    }
}
