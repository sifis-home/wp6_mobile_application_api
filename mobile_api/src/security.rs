//! Security Related Utilities
//!
//! This module contains a Secure Random Numer Generator SRNG, which allows generating of
//! cryptographically secure random bytes, AuthorizationKey, and UUIDv7.
//!
//! For the UUIDv7, we need UNIX time in milliseconds, which is done with the get_unix_time_ms.

use crate::error::{Error, Result};
use ring::rand::{SecureRandom, SystemRandom};
use schemars::gen::SchemaGenerator;
use schemars::schema::{Metadata, Schema, StringValidation};
use schemars::JsonSchema;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Display, Formatter, LowerHex, UpperHex};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// This function returns a Unix timestamp in milliseconds.
///
/// When testing with the Miri, the function always returns the test pattern of `0x0155_5555_5555`.
/// The test pattern is used because a real-time clock is unavailable when testing with Miri with
/// the isolation.
///
/// The function returns an error if the system time is before the Unix epoch.
pub fn get_unix_time_ms() -> Result<u128> {
    if cfg!(miri) {
        Ok(0x0155_5555_5555)
    } else {
        Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())
    }
}

/// SecurityKeys are stored as bytes into memory
pub type KeyBytes = [u8; 32];

/// 256-bit security key
///
/// These are used as authorization key for checking if client can use HTTP API endpoints and as
/// shared key between DHT clients.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SecurityKey(KeyBytes);

/// Common reason for wrong SecurityKey when parsing from the string
const WRONG_LENGTH_ERROR: &str = "key data length is incorrect";

impl SecurityKey {
    /// Create new security key
    ///
    /// This function creates SRNG and uses it to generate new random key.
    /// Calling [SRNG::generate_key] directly is more efficient.
    pub fn new() -> Result<SecurityKey> {
        SRNG::new().generate_key()
    }

    /// Return a slice of 32 bytes containing the value
    ///
    /// This method borrows the underlying value.
    pub const fn as_bytes(&self) -> &KeyBytes {
        &self.0
    }

    /// Returns two 128bit unsigned values containing the key
    ///
    /// The first byte from key is also most significant byte of the first u128.
    ///
    /// # Example
    /// ```rust
    /// use mobile_api::security::SecurityKey;
    /// let key = SecurityKey::from_hex(concat!(
    ///     "000102030405060708090a0b0c0d0e0f", // First half
    ///     "f0e0d0c0b0a090807060504030201000", // Second half
    /// )).unwrap();
    /// let (a, b) = key.as_u128_pair();
    /// assert_eq!(a, 0x000102030405060708090a0b0c0d0e0f);  // First half
    /// assert_eq!(b, 0xf0e0d0c0b0a090807060504030201000);  // Second half
    /// ```
    pub const fn as_u128_pair(&self) -> (u128, u128) {
        (
            ((self.as_bytes()[0] as u128) << 120
                | (self.as_bytes()[1] as u128) << 112
                | (self.as_bytes()[2] as u128) << 104
                | (self.as_bytes()[3] as u128) << 96
                | (self.as_bytes()[4] as u128) << 88
                | (self.as_bytes()[5] as u128) << 80
                | (self.as_bytes()[6] as u128) << 72
                | (self.as_bytes()[7] as u128) << 64
                | (self.as_bytes()[8] as u128) << 56
                | (self.as_bytes()[9] as u128) << 48
                | (self.as_bytes()[10] as u128) << 40
                | (self.as_bytes()[11] as u128) << 32
                | (self.as_bytes()[12] as u128) << 24
                | (self.as_bytes()[13] as u128) << 16
                | (self.as_bytes()[14] as u128) << 8
                | (self.as_bytes()[15] as u128)),
            ((self.as_bytes()[16] as u128) << 120
                | (self.as_bytes()[17] as u128) << 112
                | (self.as_bytes()[18] as u128) << 104
                | (self.as_bytes()[19] as u128) << 96
                | (self.as_bytes()[20] as u128) << 88
                | (self.as_bytes()[21] as u128) << 80
                | (self.as_bytes()[22] as u128) << 72
                | (self.as_bytes()[23] as u128) << 64
                | (self.as_bytes()[24] as u128) << 56
                | (self.as_bytes()[25] as u128) << 48
                | (self.as_bytes()[26] as u128) << 40
                | (self.as_bytes()[27] as u128) << 32
                | (self.as_bytes()[28] as u128) << 24
                | (self.as_bytes()[29] as u128) << 16
                | (self.as_bytes()[30] as u128) << 8
                | (self.as_bytes()[31] as u128)),
        )
    }

    /// Create a key from base64 string
    pub fn from_base64(string: &str) -> Result<SecurityKey> {
        match base64::decode(string)?.as_slice().try_into() {
            Ok(bytes) => Ok(SecurityKey(bytes)),
            Err(_) => Err(Error::security_key_wrong(WRONG_LENGTH_ERROR)),
        }
    }

    /// Create a key from the bytes
    pub const fn from_bytes(bytes: KeyBytes) -> SecurityKey {
        SecurityKey(bytes)
    }

    /// Crate a key from the hex string
    ///
    /// The hex string is expected to be exactly 64 characters long. Hex values can use lowercase,
    /// uppercase, or mix them.
    ///
    /// The function returns an error if the given string is not the correct length or has invalid
    /// characters.
    pub fn from_hex(hex: &str) -> Result<SecurityKey> {
        if hex.len() != 64 {
            return Err(Error::security_key_wrong(WRONG_LENGTH_ERROR));
        }
        let mut bytes = [0u8; 32];
        let mut it = bytes.iter_mut();
        for i in (0..64).step_by(2) {
            *it.next().unwrap() = u8::from_str_radix(&hex[i..i + 2], 16)?;
        }
        Ok(SecurityKey::from_bytes(bytes))
    }

    /// Create a key from string
    ///
    /// Given string can be either hex string or base64 encoded.
    ///
    ///
    /// # Example
    /// ```rust
    /// use mobile_api::security::SecurityKey;
    /// let expected_key = SecurityKey::from_bytes([
    ///     0xf0, 0xe1, 0xd2, 0xc3, 0xb4, 0xa5, 0x96, 0x87, 0x78, 0x69, 0x5a, 0x4b, 0x3c, 0x2d,
    ///     0x1e, 0x0f, 0x0f, 0x1e, 0x2d, 0x3c, 0x4b, 0x5a, 0x69, 0x78, 0x87, 0x96, 0xa5, 0xb4,
    ///     0xc3, 0xd2, 0xe1, 0xf0,
    /// ]);
    /// let key_from_hex = SecurityKey::from_string(
    ///     "f0e1d2c3b4a5968778695a4b3c2d1e0f0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap();
    /// let key_from_base64 = SecurityKey::from_string(
    ///     "8OHSw7Sllod4aVpLPC0eDw8eLTxLWml4h5altMPS4fA=").unwrap();
    /// assert_eq!(key_from_hex, expected_key);
    /// assert_eq!(key_from_base64, expected_key);
    /// ```
    pub fn from_string(string: &str) -> Result<SecurityKey> {
        if string.len() == 64 {
            if let Ok(key) = SecurityKey::from_hex(string) {
                return Ok(key);
            }
        }
        if let Ok(key) = SecurityKey::from_base64(string) {
            return Ok(key);
        }
        Err(Error::security_key_wrong(
            "the key provided was not a suitable hex or base64 string",
        ))
    }

    /// Converting key to hex string
    ///
    /// The upper parameter allows choosing between lowercase(false) and uppercase(true).
    pub fn hex(&self, upper: bool) -> String {
        /// For mapping half-bytes to uppercase characters
        const UPPER: [char; 16] = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
        ];

        /// For mapping half-bytes to lowercase characters
        const LOWER: [char; 16] = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
        ];

        let mapper = if upper { &UPPER } else { &LOWER };
        let mut hex_string = String::with_capacity(64);
        for byte in &self.0 {
            hex_string.push(mapper[(byte >> 4) as usize]);
            hex_string.push(mapper[(byte & 0x0F) as usize]);
        }
        hex_string
    }

    /// Consumes self and returns the underlying byte values
    pub const fn into_bytes(self) -> KeyBytes {
        self.0
    }

    /// Tests if the key is null (all zeros)
    pub fn is_null(&self) -> bool {
        self.as_bytes() == &[0x00; 32]
    }
}

impl Debug for SecurityKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.hex(false))
    }
}

impl<'de> Deserialize<'de> for SecurityKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// Helper function to map errors
        fn de_error<E: de::Error>(e: Error) -> E {
            E::custom(format_args!("SecurityKey parsing failed: {}", e))
        }

        if deserializer.is_human_readable() {
            /// For converting human readable str to SecurityKey object
            struct SecurityKeyVisitor;

            impl<'vi> de::Visitor<'vi> for SecurityKeyVisitor {
                type Value = SecurityKey;

                fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                    write!(formatter, "64 hex characters")
                }

                fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    SecurityKey::from_hex(v).map_err(de_error)
                }
            }

            deserializer.deserialize_str(SecurityKeyVisitor)
        } else {
            /// For converting bytes to SecurityKey object
            struct SecurityKeyBytesVisitor;

            impl<'vi> de::Visitor<'vi> for SecurityKeyBytesVisitor {
                type Value = SecurityKey;

                fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                    write!(formatter, "32 bytes")
                }

                fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    if v.len() == 32 {
                        let mut key_bytes = [0u8; 32];
                        key_bytes[..].copy_from_slice(v);
                        Ok(SecurityKey::from_bytes(key_bytes))
                    } else {
                        Err(de_error(Error::security_key_wrong(WRONG_LENGTH_ERROR)))
                    }
                }
            }

            deserializer.deserialize_bytes(SecurityKeyBytesVisitor)
        }
    }
}

impl Display for SecurityKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hex(false))
    }
}

impl LowerHex for SecurityKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hex(false))
    }
}

impl JsonSchema for SecurityKey {
    fn schema_name() -> String {
        String::from("SecurityKey")
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = String::json_schema(gen).into_object();
        let metadata = Metadata {
            description: Some("A 256-bit key as a hex string".to_string()),
            examples: vec![
                "f0e1d2c3b4a5968778695a4b3c2d1e0f0f1e2d3c4b5a69788796a5b4c3d2e1f0"
                    .to_string()
                    .into(),
            ],
            ..Default::default()
        };
        schema.metadata = Some(Box::new(metadata));
        let string = StringValidation {
            max_length: Some(64),
            min_length: Some(64),
            pattern: Some("^[0-9a-fA-F]{64}$".to_string()),
        };
        schema.string = Some(Box::new(string));
        schema.into()
    }
}

impl Serialize for SecurityKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(self.hex(false).as_str())
        } else {
            serializer.serialize_bytes(self.as_bytes())
        }
    }
}

impl UpperHex for SecurityKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hex(true))
    }
}

/// Secure Random Number Generator
///
/// This struct uses a ring crate to generate cryptographically secure random bytes. A few
/// convenience functions were added to generate AuthorizationKey and UUIDv7 easily.
///
/// # Example
///
/// ```rust
/// use mobile_api::security::SRNG;
///
/// let srng = SRNG::new();
///
/// // Generate SecureKey
/// let key = srng.generate_key().unwrap();
///
/// // Generate Uuid
/// let uuid = srng.generate_uuid().unwrap();
///
/// // Generate random bytes
/// let mut bytes = [0u8; 16];
/// srng.fill(&mut bytes).unwrap();
/// ```
pub struct SRNG {
    /// Using SystemRandom from the ring crate to generate secure random numbers
    rng: SystemRandom,
}

impl SRNG {
    /// Construct new Random Number Generator
    pub fn new() -> SRNG {
        SRNG {
            rng: SystemRandom::new(),
        }
    }

    /// Fill buffer with random bytes
    pub fn fill(&self, buf: &mut [u8]) -> Result<()> {
        self.rng.fill(buf)?;
        Ok(())
    }

    /// Generating secure random 256-bit key
    pub fn generate_key(&self) -> Result<SecurityKey> {
        let mut key = [0u8; 32];
        self.fill(&mut key)?;
        Ok(SecurityKey::from_bytes(key))
    }

    /// Generating UUIDv7 for Smart Device
    ///
    /// The UUID crate has UUIDv7 as an unstable feature because new versions are still draft.
    /// We did not want to enable an unstable feature, but the UUIDv7 is suitable for our purposes,
    /// so we implemented this function.
    ///
    /// UUID version 7 fields and bit layout:
    ///
    /// ```text
    ///  0                   1                   2                   3
    ///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |                           unix_ts_ms                          |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |          unix_ts_ms           |  ver  |       rand_a          |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |var|                        rand_b                             |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |                            rand_b                             |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// ```
    ///
    /// | Field      | Bits | Description                                         |
    /// | ---------- | ---- | --------------------------------------------------- |
    /// | unix_ts_ms | 48   | Timestamp as milliseconds since the UNIX_EPOCH      |
    /// | ver        | 4    | Version number                                      |
    /// | rand_a     | 12   | Random bits                                         |
    /// | var        | 2    | The variant field determines the layout of the UUID |
    /// | rand_b     | 62   | Random bits                                         |
    pub fn generate_uuid(&self) -> Result<Uuid> {
        // First 48 bits are unix time in milliseconds
        let mut uuid = get_unix_time_ms()? << 80;

        // Randomizing rest of the bits
        let mut bytes = [0u8; 16];
        self.fill(&mut bytes[6..])?;
        uuid |= u128::from_be_bytes(bytes);

        // Setting UUID version 7 bits
        uuid &= 0xFFFFFFFF_FFFF_0FFF_3FFF_FFFFFFFFFFFF;
        uuid |= 0x00000000_0000_7000_8000_000000000000;

        Ok(Uuid::from_u128(uuid))
    }
}

impl Default for SRNG {
    /// Construct new Random Number Generator
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::schema::{InstanceType, SingleOrVec};
    use schemars::schema_for;

    const TEST_KEY_BYTES: KeyBytes = [
        0xf0, 0xe1, 0xd2, 0xc3, 0xb4, 0xa5, 0x96, 0x87, 0x78, 0x69, 0x5a, 0x4b, 0x3c, 0x2d, 0x1e,
        0x0f, 0x0f, 0x1e, 0x2d, 0x3c, 0x4b, 0x5a, 0x69, 0x78, 0x87, 0x96, 0xa5, 0xb4, 0xc3, 0xd2,
        0xe1, 0xf0,
    ];

    const TEST_KEY_HEX: &str = "f0e1d2c3b4a5968778695a4b3c2d1e0f0f1e2d3c4b5a69788796a5b4c3d2e1f0";

    const TEST_KEY_BASE64: &str = "8OHSw7Sllod4aVpLPC0eDw8eLTxLWml4h5altMPS4fA=";

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
        assert_eq!(display, TEST_KEY_HEX);
        assert_eq!(debug, format!("\"{}\"", TEST_KEY_HEX));
        assert_eq!(lower_hex, TEST_KEY_HEX);
        assert_eq!(upper_hex, TEST_KEY_HEX.to_uppercase());
    }

    #[test]
    fn test_security_key_from_hex() {
        // Wrong size should cause error
        let result = SecurityKey::from_hex("00");
        assert!(result.is_err());

        // Invalid characters should cause error
        let result = SecurityKey::from_hex(
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        );
        assert!(result.is_err());

        // Valid string should give correct key (both lower and upper case hex should be okay)
        let key = SecurityKey::from_hex(TEST_KEY_HEX).unwrap();
        assert_eq!(key.as_bytes(), &TEST_KEY_BYTES);
    }

    #[test]
    fn test_security_key_from_string() {
        // Valid strings
        let key_from_hex = SecurityKey::from_string(TEST_KEY_HEX).unwrap();
        let key_from_base64 = SecurityKey::from_string(TEST_KEY_BASE64).unwrap();
        assert_eq!(TEST_KEY, key_from_hex);
        assert_eq!(TEST_KEY, key_from_base64);

        // Invalid strings
        assert!(SecurityKey::from_string(
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        )
        .is_err(),);
        assert!(SecurityKey::from_string("8OHSw7Sllod4aVpLPC0eDw==").is_err());
    }

    #[test]
    fn test_security_key_hex() {
        assert_eq!(TEST_KEY.hex(false), TEST_KEY_HEX);
        assert_eq!(TEST_KEY.hex(true), TEST_KEY_HEX.to_uppercase());
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
        assert!(
            error_message.starts_with("SecurityKey parsing failed: key data length is incorrect")
        );

        // Invalid characters in JSON should cause error
        let json = r#""----------------------------------------------------------------""#;
        let result = serde_json::from_str::<SecurityKey>(json);
        assert!(result.is_err());
        let error_message = format!("{}", result.err().unwrap());
        assert!(
            error_message.starts_with("SecurityKey parsing failed: invalid digit found in string")
        );

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
}
