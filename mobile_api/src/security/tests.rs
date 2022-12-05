use super::*;
use schemars::schema::{InstanceType, SingleOrVec};
use schemars::schema_for;

const TEST_KEY_BYTES: KeyBytes = [
    0xf0, 0xe1, 0xd2, 0xc3, 0xb4, 0xa5, 0x96, 0x87, 0x78, 0x69, 0x5a, 0x4b, 0x3c, 0x2d, 0x1e, 0x0f,
    0x0f, 0x1e, 0x2d, 0x3c, 0x4b, 0x5a, 0x69, 0x78, 0x87, 0x96, 0xa5, 0xb4, 0xc3, 0xd2, 0xe1, 0xf0,
];

const TEST_KEY: SecurityKey = SecurityKey::from_bytes(TEST_KEY_BYTES);

#[test]
fn test_get_unix_time_ms() {
    let result = get_unix_time_ms();
    assert!(result.is_ok());

    if cfg!(miri) {
        assert_eq!(get_unix_time_ms().unwrap(), 0x0155_5555_5555);
    }
}

#[test]
fn test_security_key_new() {
    // SRNG is well tested in test_srng_generate_key, here we just check that we get random key
    let key = SecurityKey::new().unwrap();
    assert!(!key.is_null())
}

#[test]
fn test_security_key_as_bytes() {
    assert_eq!(TEST_KEY.as_bytes(), &TEST_KEY_BYTES);
}

#[test]
fn test_security_key_as_u128_pair() {
    let (a, b) = TEST_KEY.as_u128_pair();
    assert_eq!(a, 0xf0e1_d2c3_b4a5_9687_7869_5a4b_3c2d_1e0f);
    assert_eq!(b, 0x0f1e_2d3c_4b5a_6978_8796_a5b4_c3d2_e1f0);
}

#[test]
fn test_security_key_formatting() {
    let display = format!("{}", TEST_KEY);
    let debug = format!("{:?}", TEST_KEY);
    let lower_hex = format!("{:x}", TEST_KEY);
    let upper_hex = format!("{:X}", TEST_KEY);
    assert_eq!(
        display,
        "f0e1d2c3b4a5968778695a4b3c2d1e0f0f1e2d3c4b5a69788796a5b4c3d2e1f0"
    );
    assert_eq!(
        debug,
        r#""f0e1d2c3b4a5968778695a4b3c2d1e0f0f1e2d3c4b5a69788796a5b4c3d2e1f0""#
    );
    assert_eq!(
        lower_hex,
        "f0e1d2c3b4a5968778695a4b3c2d1e0f0f1e2d3c4b5a69788796a5b4c3d2e1f0"
    );
    assert_eq!(
        upper_hex,
        "F0E1D2C3B4A5968778695A4B3C2D1E0F0F1E2D3C4B5A69788796A5B4C3D2E1F0"
    );
}

#[test]
fn test_security_key_from_hex() {
    // Wrong size should cause error
    let result = SecurityKey::from_hex("00");
    assert!(result.is_err());

    // Invalid characters should cause error
    let result =
        SecurityKey::from_hex("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    assert!(result.is_err());

    // Valid string should give correct key (both lower and upper case hex should be okay)
    let hex = "f0e1d2c3b4a5968778695a4b3c2d1e0f0F1E2D3C4B5A69788796A5B4C3D2E1F0";
    let key = SecurityKey::from_hex(hex).unwrap();
    assert_eq!(key.as_bytes(), &TEST_KEY_BYTES);
}

#[test]
fn test_security_key_hex() {
    assert_eq!(
        TEST_KEY.hex(false),
        "f0e1d2c3b4a5968778695a4b3c2d1e0f0f1e2d3c4b5a69788796a5b4c3d2e1f0"
    );
    assert_eq!(
        TEST_KEY.hex(true),
        "F0E1D2C3B4A5968778695A4B3C2D1E0F0F1E2D3C4B5A69788796A5B4C3D2E1F0"
    );
}

#[test]
fn test_security_key_into_bytes() {
    assert_eq!(TEST_KEY.into_bytes(), TEST_KEY_BYTES);
}

#[test]
fn test_security_key_serde() {
    // Testing human readable with JSON
    let key_a = SecurityKey::new().unwrap();
    let json = serde_json::to_string(&key_a).unwrap();
    let key_b = serde_json::from_str::<SecurityKey>(&json).unwrap();
    assert_eq!(key_a, key_b);

    // Invalid length JSON should cause error
    let json = r#""F0E1D2C3B4A5968778695A4B3C2D1E0F""#;
    let result = serde_json::from_str::<SecurityKey>(json);
    assert!(result.is_err());
    let error_message = format!("{}", result.err().unwrap());
    assert!(error_message.starts_with("SecurityKey parsing failed: key data length is incorrect"));

    // Invalid characters in JSON should cause error
    let json = r#""----------------------------------------------------------------""#;
    let result = serde_json::from_str::<SecurityKey>(json);
    assert!(result.is_err());
    let error_message = format!("{}", result.err().unwrap());
    assert!(error_message.starts_with("SecurityKey parsing failed: invalid digit found in string"));

    // Wrong type should cause error
    let json = "true";
    let result = serde_json::from_str::<SecurityKey>(json);
    assert!(result.is_err());
    let error_message = format!("{}", result.err().unwrap());
    assert!(error_message.contains("64 hex characters"));

    // Testing binary with MessagePack
    let buf = rmp_serde::to_vec(&key_a).unwrap();
    let key_b = rmp_serde::from_slice(&buf).unwrap();
    assert_eq!(key_a, key_b);

    // Wrong byte count should cause error
    let result = rmp_serde::from_slice::<SecurityKey>(&[0xc4, 0x04, 0x00, 0x00, 0x00, 0x00]);
    assert!(result.is_err());
    let error_message = format!("{}", result.err().unwrap());
    assert_eq!(
        error_message,
        "SecurityKey parsing failed: key data length is incorrect"
    );

    // Wrong type should cause error
    let result = rmp_serde::from_slice::<SecurityKey>(&[0xa4, 0x54, 0x65, 0x73, 0x74]);
    assert!(result.is_err());
    let error_message = format!("{}", result.err().unwrap());
    assert!(error_message.contains("32 bytes"));
}

#[test]
fn test_security_key_schema() {
    let schema = schema_for!(SecurityKey).schema;

    // Should have valid metadata
    let metadata = schema.metadata.unwrap();
    assert_eq!(metadata.title.unwrap(), "SecurityKey");
    assert_eq!(
        metadata.description.unwrap(),
        "A 256-bit key as a hex string"
    );

    // Should have Single String instance type
    let instance_type = schema.instance_type.unwrap();
    let expected_type = SingleOrVec::Single(Box::new(InstanceType::String));
    assert_eq!(instance_type, expected_type);

    // Should have string validation for 64 character long hexadecimal
    let string = schema.string.unwrap();
    assert_eq!(string.max_length.unwrap(), 64);
    assert_eq!(string.min_length.unwrap(), 64);
    assert_eq!(string.pattern.unwrap(), "^[0-9a-fA-F]{64}$");
}

#[test]
fn test_srng_fill() {
    let mut buffer_a = [0u8; 256];
    let mut buffer_b = [0u8; 256];
    let srng = SRNG::default();
    srng.fill(&mut buffer_a).unwrap();
    srng.fill(&mut buffer_b).unwrap();
    assert_ne!(buffer_a, [0u8; 256]);
    assert_ne!(buffer_b, [0u8; 256]);
    assert_ne!(buffer_a, buffer_b);
}

#[test]
fn test_srng_generate_key() {
    let srng = SRNG::new();
    // We generate bunch of random keys and count each bit
    let mut bit_count = [0u32; 256];
    let key_count = 1024;
    for _ in 0..key_count {
        let key = srng.generate_key().unwrap();
        let mut it = bit_count.iter_mut();
        for byte in key.as_bytes() {
            for bit in (0..8).rev() {
                *it.next().unwrap() += ((byte & (1 << bit)) >> bit) as u32;
            }
        }
    }

    // All bit counts should be about half of the key_count. However, since these are random
    // numbers, we accept the test passed as long as each is between 25% and 75%.
    let acceptable_min = key_count / 4;
    let acceptable_max = acceptable_min * 3;
    for count in bit_count {
        assert!(count >= acceptable_min);
        assert!(count <= acceptable_max)
    }
}

#[test]
fn test_srng_generate_uuid() {
    // Get current system time to compare results
    let unix_ts_start = get_unix_time_ms().unwrap();

    // UUID generation should work
    let srng = SRNG::new();
    let uuid_a = srng.generate_uuid().unwrap();
    let uuid_b = srng.generate_uuid().unwrap();

    // Generated identifiers should be different
    assert_ne!(uuid_a, uuid_b);

    // Both should have version number 7
    assert_eq!(uuid_a.get_version_num(), 7);
    assert_eq!(uuid_b.get_version_num(), 7);

    // Both should have variant bits as 0b10..
    assert_eq!(uuid_a.as_fields().3[0] & 0b1100_0000, 0b1000_0000);
    assert_eq!(uuid_b.as_fields().3[0] & 0b1100_0000, 0b1000_0000);

    // Both should have timestamp greater or equal to timestamp we created at start
    let unix_ts_a = uuid_a.as_u128() >> 80;
    let unix_ts_b = uuid_b.as_u128() >> 80;
    assert!(unix_ts_a >= unix_ts_start);
    assert!(unix_ts_b >= unix_ts_start);

    // B should have greater or equal timestamp with A
    assert!(unix_ts_b >= unix_ts_a);
}
