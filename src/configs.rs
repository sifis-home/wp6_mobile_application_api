use crate::error::Result;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

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
    /// This function uses OpenSSL to generate cryptographically strong pseudo-random key
    pub fn generate_authorization_key() -> Result<AuthorizationKey> {
        let mut authorization_key = [0u8; 32];
        openssl::rand::rand_bytes(&mut authorization_key)?;
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
    pub fn generate_uuid() -> Result<Uuid> {
        // First 48 bits are unix time in milliseconds
        let mut uuid = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() << 80;

        // Randomizing rest of the bits
        let mut bytes = [0u8; 16];
        openssl::rand::rand_bytes(&mut bytes[6..])?;
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

    #[test]
    fn generate_authorization_key() {
        let auth_key_a = DeviceInfo::generate_authorization_key();
        let auth_key_b = DeviceInfo::generate_authorization_key();
        assert!(auth_key_a.is_ok());
        assert!(auth_key_b.is_ok());
        assert_ne!(auth_key_a.unwrap(), auth_key_b.unwrap());
    }

    #[test]
    fn generate_uuid() {
        // Get current system time to compare results
        let unix_ts_start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        // UUID generation should work
        let uuid_a = DeviceInfo::generate_uuid();
        let uuid_b = DeviceInfo::generate_uuid();
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
        // a time difference that is less than 5 seconds.
        let time_diff_a = unix_ts_a - unix_ts_start;
        let time_diff_b = unix_ts_b - unix_ts_start;
        assert!(
            time_diff_a < 5000,
            "Expected time difference more than 5000 ms: {}",
            time_diff_a
        );
        assert!(
            time_diff_b < 5000,
            "Expected time difference more than 5000 ms: {}",
            time_diff_b
        );
    }
}
