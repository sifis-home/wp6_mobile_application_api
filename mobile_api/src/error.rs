//! Error reporting

use std::fmt;

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

    /// Convenience function for reporting errors with SecurityKey
    pub(crate) fn security_key_wrong(reason: &'static str) -> Error {
        Error(Box::new(ErrorKind::SecurityKeyWrong(reason)))
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
            ErrorKind::Base64DecodeError(ref err) => err.fmt(f),
            ErrorKind::IoError(ref err) => err.fmt(f),
            ErrorKind::NumParseIntError(ref err) => err.fmt(f),
            ErrorKind::RngError(ref err) => err.fmt(f),
            ErrorKind::SecurityKeyWrong(reason) => reason.fmt(f),
            ErrorKind::SerdeJson(ref err) => err.fmt(f),
            ErrorKind::TimeError(ref err) => err.fmt(f),
        }
    }
}

/// The specific type of an error
#[derive(Debug)]
pub enum ErrorKind {
    /// Base64 decode error
    Base64DecodeError(base64::DecodeError),
    /// Standard I/O errors
    IoError(std::io::Error),
    /// Error while parsing integer value from str
    NumParseIntError(std::num::ParseIntError),
    /// Unspecified error from the ring crate
    RngError(ring::error::Unspecified),
    /// Error when converting string to SecurityKey
    SecurityKeyWrong(&'static str),
    /// For JSON serialization errors
    SerdeJson(serde_json::Error),
    /// Error with the time
    TimeError(std::time::SystemTimeError),
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Self {
        Error::new(ErrorKind::Base64DecodeError(err))
    }
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

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::SecurityKey;

    #[test]
    fn test_io_error() {
        let io_error_source = std::io::Error::new(std::io::ErrorKind::Other, "example error");
        let io_error = Error::from(io_error_source);
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
    fn test_num_parse_int_error() {
        let parse_error_source = "x".parse::<u8>().err().unwrap();
        let parse_error = Error::from(parse_error_source);
        let parse_error_debug = format!("{:?}", parse_error);
        let parse_error_display = format!("{}", parse_error);
        assert_eq!(
            parse_error_debug,
            "Error(NumParseIntError(ParseIntError { kind: InvalidDigit }))"
        );
        assert_eq!(parse_error_display, "invalid digit found in string");
        assert!(matches!(parse_error.kind(), ErrorKind::NumParseIntError(_)));
        assert!(matches!(
            parse_error.into_kind(),
            ErrorKind::NumParseIntError(_)
        ));
    }

    #[test]
    fn test_rng_error() {
        let rng_error_source = ring::error::Unspecified;
        let rng_error = Error::from(rng_error_source);
        let rng_error_debug = format!("{:?}", rng_error);
        let rng_error_display = format!("{}", rng_error);
        assert_eq!(rng_error_debug, "Error(RngError(Unspecified))");
        assert_eq!(rng_error_display, "ring::error::Unspecified");
        assert!(matches!(rng_error.kind(), ErrorKind::RngError(_)));
        assert!(matches!(rng_error.into_kind(), ErrorKind::RngError(_)));
    }

    #[test]
    fn test_security_key_wrong_size_error() {
        let key_error = SecurityKey::from_hex("_").err().unwrap();
        let key_error_debug = format!("{:?}", key_error);
        let key_error_display = format!("{}", key_error);
        assert_eq!(
            key_error_debug,
            "Error(SecurityKeyWrong(\"key data length is incorrect\"))"
        );
        assert_eq!(key_error_display, "key data length is incorrect");
        assert!(matches!(key_error.kind(), ErrorKind::SecurityKeyWrong(_)));
        assert!(matches!(
            key_error.into_kind(),
            ErrorKind::SecurityKeyWrong(_)
        ));
    }

    #[test]
    fn test_serde_json_error() {
        let json_error_source = serde_json::from_str::<String>("").err().unwrap();
        let expected_debug = format!("Error(SerdeJson({:?}))", json_error_source);
        let expected_display = format!("{}", json_error_source);
        let json_error = Error::from(json_error_source);
        let json_error_debug = format!("{:?}", json_error);
        let json_error_display = format!("{}", json_error);
        assert_eq!(json_error_debug, expected_debug);
        assert_eq!(json_error_display, expected_display);
        assert!(matches!(json_error.kind(), ErrorKind::SerdeJson(_)));
        assert!(matches!(json_error.into_kind(), ErrorKind::SerdeJson(_)));
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
        let time_error = Error::from(time_error_source);
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
