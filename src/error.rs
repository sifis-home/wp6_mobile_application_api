//! Error reporting

use std::fmt;

/// A type alias for `Result<T, mobile_api::error::Error>`
pub type Result<T> = std::result::Result<T, Error>;

/// Various errors can occur when using the mobile_api crate
///
/// This Error is a container for boxed ErrorKind. From traits are implemented for known Errors,
/// and the Display trait is implemented to allow formatting of the error messages.
///
/// The `kind()` and `into_kind()` allow accessing the actual error object.
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
            ErrorKind::RngError(ref err) => err.fmt(f),
            ErrorKind::TimeError(ref err) => err.fmt(f),
        }
    }
}

/// The specific type of an error
#[derive(Debug)]
pub enum ErrorKind {
    /// Standard I/O errors
    IoError(std::io::Error),
    /// Unspecified error from the ring crate
    RngError(ring::error::Unspecified),
    /// Error with the time
    TimeError(std::time::SystemTimeError),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::new(ErrorKind::IoError(err))
    }
}

impl From<ring::error::Unspecified> for Error {
    fn from(err: ring::error::Unspecified) -> Self {
        Error::new(ErrorKind::RngError(err))
    }
}

impl From<std::time::SystemTimeError> for Error {
    fn from(err: std::time::SystemTimeError) -> Self {
        Error::new(ErrorKind::TimeError(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error() {
        let io_error_source = std::io::Error::new(std::io::ErrorKind::Other, "example error");
        let io_error: Error = io_error_source.into();
        let io_error_debug = format!("{:?}", io_error);
        let io_error_display = format!("{}", io_error);
        assert_eq!(
            io_error_debug,
            "Error(IoError(Custom { kind: Other, error: \"example error\" }))"
        );
        assert_eq!(io_error_display, "example error");
        assert!(matches!(io_error.kind(), ErrorKind::IoError(_)));
        assert!(matches!(io_error.into_kind(), ErrorKind::IoError(_)));
    }

    #[test]
    fn test_rng_error() {
        let rng_error_source = ring::error::Unspecified;
        let rng_error: Error = rng_error_source.into();
        let rng_error_debug = format!("{:?}", rng_error);
        let rng_error_display = format!("{}", rng_error);
        assert_eq!(rng_error_debug, "Error(RngError(Unspecified))");
        assert_eq!(rng_error_display, "ring::error::Unspecified");
        assert!(matches!(rng_error.kind(), ErrorKind::RngError(_)));
        assert!(matches!(rng_error.into_kind(), ErrorKind::RngError(_)));
    }

    #[test]
    #[cfg_attr(miri, ignore)] // SystemTime does not work with miri
    fn test_time_error() {
        use std::thread::sleep;
        use std::time::{Duration, SystemTime};

        let time_a = SystemTime::now();
        sleep(Duration::from_millis(10));
        let time_b = SystemTime::now();
        let time_error_source = time_a.duration_since(time_b).err().unwrap();
        let time_error: Error = time_error_source.into();
        let time_error_debug = format!("{:?}", time_error);
        let time_error_display = format!("{}", time_error);

        assert!(time_error_debug.starts_with("Error(TimeError(SystemTimeError("));
        assert!(time_error_debug.ends_with(")))"));
        assert_eq!(
            time_error_display,
            "second time provided was later than self"
        );
        assert!(matches!(time_error.kind(), ErrorKind::TimeError(_)));
        assert!(matches!(time_error.into_kind(), ErrorKind::TimeError(_)));
    }
}
