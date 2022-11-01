use crate::error::Result;

use ring::rand::SecureRandom;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// Helper functions
//--------------------------------------------------------------------------------------------------

fn get_unix_time_ms() -> Result<u128> {
    if cfg!(miri) {
        // REALTIME is not available when running with Miri.
        // Therefore, we return suitable test pattern instead.
        Ok(0x0155_5555_5555)
    } else {
        Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())
    }
}

// 256-bit authorization key
//--------------------------------------------------------------------------------------------------

pub type AuthorizationKey = [u8; 32];

// Device information
//--------------------------------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceInfo {
    authorization_key: AuthorizationKey,
    private_key_file: PathBuf,
    product_name: String,
    uuid: Uuid,
}

impl DeviceInfo {
    /// Generating secure random authorization key
    ///
    /// This function uses ring crate to generate secure random key
    ///
    /// # Example
    ///
    /// ```rust
    /// use ring::rand::SystemRandom;
    /// use mobile_api::configs::DeviceInfo;
    ///
    /// let rng = SystemRandom::new();
    /// let key = DeviceInfo::generate_authorization_key(&rng).unwrap();
    /// ```
    ///
    /// **Note:** An application should create a single SystemRandom and then use it for all
    /// randomness generation. Besides being more efficient, this also helps document where
    /// non-deterministic (random) outputs occur.
    pub fn generate_authorization_key(rng: &dyn SecureRandom) -> Result<AuthorizationKey> {
        let mut authorization_key = [0u8; 32];
        rng.fill(&mut authorization_key)?;
        Ok(authorization_key)
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
    ///
    /// This function uses ring crate to generate secure random bytes
    ///
    /// # Example
    ///
    /// ```rust
    /// use ring::rand::SystemRandom;
    /// use mobile_api::configs::DeviceInfo;
    ///
    /// let rng = SystemRandom::new();
    /// let uuid = DeviceInfo::generate_uuid(&rng).unwrap();
    /// ```
    ///
    /// **Note:** An application should create a single SystemRandom and then use it for all
    /// randomness generation. Besides being more efficient, this also helps document where
    /// non-deterministic (random) outputs occur.
    pub fn generate_uuid(rng: &dyn SecureRandom) -> Result<Uuid> {
        // First 48 bits are unix time in milliseconds
        let mut uuid = get_unix_time_ms()? << 80;

        // Randomizing rest of the bits
        let mut bytes = [0u8; 16];
        rng.fill(&mut bytes[6..])?;
        uuid |= u128::from_be_bytes(bytes);

        // Setting UUID version 7 bits
        uuid &= 0xFFFFFFFF_FFFF_0FFF_3FFF_FFFFFFFFFFFF;
        uuid |= 0x00000000_0000_7000_8000_000000000000;

        Ok(Uuid::from_u128(uuid))
    }
}

// Unit tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ring::rand::SystemRandom;

    #[test]
    fn generate_authorization_key() {
        let rng = SystemRandom::new();
        let auth_key_a = DeviceInfo::generate_authorization_key(&rng);
        let auth_key_b = DeviceInfo::generate_authorization_key(&rng);
        assert!(auth_key_a.is_ok());
        assert!(auth_key_b.is_ok());
        assert_ne!(auth_key_a.unwrap(), auth_key_b.unwrap());
    }

    #[test]
    fn generate_uuid() {
        // Get current system time to compare results
        let unix_ts_start = get_unix_time_ms().unwrap();

        // UUID generation should work
        let rng = SystemRandom::new();
        let uuid_a = DeviceInfo::generate_uuid(&rng);
        let uuid_b = DeviceInfo::generate_uuid(&rng);
        assert!(uuid_a.is_ok());
        assert!(uuid_b.is_ok());

        // Generated identifiers should be different
        let uuid_a = uuid_a.unwrap();
        let uuid_b = uuid_b.unwrap();
        assert_ne!(uuid_a, uuid_b);

        // Both should have version number 7
        assert_eq!(uuid_a.get_version_num(), 7);
        assert_eq!(uuid_b.get_version_num(), 7);

        // Both should have variant bits as 0b10..
        assert_eq!(uuid_a.as_fields().3[0] & 0b1100_0000, 0b1000_0000);
        assert_eq!(uuid_b.as_fields().3[0] & 0b1100_0000, 0b1000_0000);

        // Both should have timestamp greater or equal to timestamp we created
        let unix_ts_a = uuid_a.as_u128() >> 80;
        let unix_ts_b = uuid_b.as_u128() >> 80;
        assert!(unix_ts_a >= unix_ts_start);
        assert!(unix_ts_b >= unix_ts_start);

        // B should have greater or equal timestamp with A
        assert!(unix_ts_b >= unix_ts_a);

        // The time difference between start time and time in UUID should generally be 0 -- 1 ms.
        // However, it can be much greater when the test is run with Valgrind. Therefore we accept
        // a time difference that is less than 1 second.
        let accepted_delay_ms = 1000;
        let time_diff_a = unix_ts_a - unix_ts_start;
        let time_diff_b = unix_ts_b - unix_ts_start;
        assert!(
            time_diff_a < accepted_delay_ms,
            "Expected time difference is more than {} ms: Difference {} ms",
            accepted_delay_ms,
            time_diff_a
        );
        assert!(
            time_diff_b < accepted_delay_ms,
            "Expected time difference is more than {} ms: Difference {} ms",
            accepted_delay_ms,
            time_diff_b
        );
    }
}
