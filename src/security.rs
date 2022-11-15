//! Security Related Utilities
//!
//! This module contains a Secure Random Numer Generator SRNG, which allows generating of
//! cryptographically secure random bytes, AuthorizationKey, and UUIDv7.
//!
//! For the UUIDv7, we need UNIX time in milliseconds, which is done with the get_unix_time_ms.

use crate::error::Result;
use ring::rand::{SecureRandom, SystemRandom};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// 256-bit authorization key
pub type AuthorizationKey = [u8; 32];

/// 256-bit shared key
pub type SharedKey = [u8; 32];

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
/// // Generate AuthorizationKey
/// let key = srng.generate_authorization_key().unwrap();
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

    /// Generating secure random authorization key
    pub fn generate_authorization_key(&self) -> Result<AuthorizationKey> {
        let mut key: AuthorizationKey = [0u8; 32];
        self.fill(&mut key)?;
        Ok(key)
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

    #[test]
    fn test_get_unix_time_ms() {
        let result = get_unix_time_ms();
        assert!(result.is_ok());

        if cfg!(miri) {
            let ts = get_unix_time_ms().unwrap();
            assert_eq!(ts, 0x0155_5555_5555);
        }
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
    fn test_srng_generate_authorization_key() {
        let srng = SRNG::new();
        let auth_key_a = srng.generate_authorization_key().unwrap();
        let auth_key_b = srng.generate_authorization_key().unwrap();
        assert_ne!(auth_key_a, auth_key_b);
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
