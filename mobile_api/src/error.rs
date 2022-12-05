//! Error reporting

use std::fmt;

#[cfg(test)]
mod tests;

/// A type alias for `Result<T, mobile_api::error::Error>`
pub type Result<T> = std::result::Result<T, Error>;

/// Various errors can occur when using the mobile_api crate
///
/// This Error is a container for boxed ErrorKind. From traits are implemented for known Errors,
/// and the Display trait is implemented to allow formatting of the error messages.
///
/// The [Error::kind] and [Error::into_kind] allow accessing the actual error object.
#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

impl Error {
    /// Create Error with known ErrorKind
    pub(crate) fn new(kind: ErrorKind) -> Error {
        Error(Box::new(kind))
    }

    /// Return the specific type of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }

    /// Unwrap this error into its underlying type.
    pub fn into_kind(self) -> ErrorKind {
        *self.0
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self.0 {
            ErrorKind::IoError(ref err) => err.fmt(f),
            ErrorKind::NumParseIntError(ref err) => err.fmt(f),
            ErrorKind::RngError(ref err) => err.fmt(f),
            ErrorKind::SecurityKeyWrongSize => write!(f, "key data length is incorrect"),
            ErrorKind::SerdeJson(ref err) => err.fmt(f),
            ErrorKind::TimeError(ref err) => err.fmt(f),
        }
    }
}

/// The specific type of an error
#[derive(Debug)]
pub enum ErrorKind {
    /// Standard I/O errors
    IoError(std::io::Error),
    /// Error while parsing integer value from str
    NumParseIntError(std::num::ParseIntError),
    /// Unspecified error from the ring crate
    RngError(ring::error::Unspecified),
    /// Invalid character count in hex string
    SecurityKeyWrongSize,
    /// For JSON serialization errors
    SerdeJson(serde_json::Error),
    /// Error with the time
    TimeError(std::time::SystemTimeError),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::new(ErrorKind::IoError(err))
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::new(ErrorKind::NumParseIntError(err))
    }
}

impl From<ring::error::Unspecified> for Error {
    fn from(err: ring::error::Unspecified) -> Self {
        Error::new(ErrorKind::RngError(err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::new(ErrorKind::SerdeJson(err))
    }
}

impl From<std::time::SystemTimeError> for Error {
    fn from(err: std::time::SystemTimeError) -> Self {
        Error::new(ErrorKind::TimeError(err))
    }
}
