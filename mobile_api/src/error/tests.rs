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
    assert_eq!(key_error_debug, "Error(SecurityKeyWrongSize)");
    assert_eq!(key_error_display, "key data length is incorrect");
    assert!(matches!(key_error.kind(), ErrorKind::SecurityKeyWrongSize));
    assert!(matches!(
        key_error.into_kind(),
        ErrorKind::SecurityKeyWrongSize
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
