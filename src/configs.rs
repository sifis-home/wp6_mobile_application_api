//! Smart Device Configuration
//!
//! This module contains DeviceInfo and DeviceConfig structures.
//!
//! The DeviceInfo handles device information usually stored to `/opt/sifis-home/device.json`.
//!
//! The DeviceConfig handles device configuration usually stored to `/opt/sifis-home/config.json`.
//! This file is missing when the Smart Device is first started, or the user has done a factory
//! reset.

use crate::error::Result;
use crate::security::{AuthorizationKey, SharedKey, SRNG};
use crate::sifis_home_path;
use std::path::PathBuf;
use uuid::Uuid;

/// Smart Device Configuration
///
/// These are settings that the owner of the device sets using the SIFIS-Home mobile application.
#[derive(Debug)]
pub struct DeviceConfig {
    /// Shared key for DHT communication, 32 bytes in hex format
    dht_shared_key: SharedKey,
    /// User-defined name for the Smart Device
    name: String,
}

impl DeviceConfig {
    /// Create a new configuration
    pub fn new(dht_shared_key: SharedKey, name: String) -> DeviceConfig {
        DeviceConfig {
            dht_shared_key,
            name,
        }
    }

    /// Borrow shared DHT key
    pub fn dht_shared_key(&self) -> &SharedKey {
        &self.dht_shared_key
    }

    /// Borrow device name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Change shared DHT key
    pub fn set_dht_shared_key(&mut self, dht_shared_key: SharedKey) {
        self.dht_shared_key = dht_shared_key;
    }

    /// Change device name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

/// Smart Device Information
///
/// This information is pre-written at the factory or can be generated when the Smart Device Mobile
/// API service is started for the first time.
///
/// Some or all of these are delivered with the device in a QR code for the mobile application to
/// scan.
#[derive(Debug)]
pub struct DeviceInfo {
    /// 256-bit authorization key in hex format. SIFIS-Home mobile application needs this key to
    /// access configuration endpoints of the Smart Device Mobile API service.
    authorization_key: AuthorizationKey,
    /// Path to DHT private key file. The sifis-dht generates key file on the first run
    private_key_file: PathBuf,
    /// Product name
    product_name: String,
    /// 128-bit UUID in standard hex format
    uuid: Uuid,
}

impl DeviceInfo {
    /// Create a new device information
    ///
    /// Product name is required, other information is automatically generated.
    pub fn new(product_name: String) -> Result<DeviceInfo> {
        let srng = SRNG::new();
        let mut private_key_file = sifis_home_path();
        private_key_file.push("private.pem");
        Ok(DeviceInfo {
            authorization_key: srng.generate_authorization_key()?,
            private_key_file,
            product_name,
            uuid: srng.generate_uuid()?,
        })
    }

    /// Borrow authorization key
    pub fn authorization_key(&self) -> &AuthorizationKey {
        &self.authorization_key
    }

    /// Borrow private key file path
    pub fn private_key_file(&self) -> &PathBuf {
        &self.private_key_file
    }

    /// Borrow product name
    pub fn product_name(&self) -> &str {
        &self.product_name
    }

    /// Borrow device UUID
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    /// Change authorization key
    ///
    /// **NOTE:** This is not good idea if authorization code is already printed as QR code for the
    /// product.
    pub fn set_authorization_key(&mut self, authorization_key: AuthorizationKey) {
        self.authorization_key = authorization_key;
    }

    /// Change private key location
    pub fn set_private_key_file(&mut self, private_key_file: PathBuf) {
        self.private_key_file = private_key_file;
    }

    /// Change product name
    pub fn set_product_name(&mut self, product_name: String) {
        self.product_name = product_name;
    }

    /// Change UUID
    pub fn set_uuid(&mut self, uuid: Uuid) {
        self.uuid = uuid;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::uuid;

    const TEST_KEY_A: [u8; 32] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
        0x1e, 0x1f,
    ];

    const TEST_KEY_B: [u8; 32] = [
        0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e,
        0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d,
        0x3e, 0x3f,
    ];

    #[test]
    fn test_device_config() {
        // Testing constructor and getters
        let mut config = DeviceConfig::new(TEST_KEY_A, "Test config".to_string());
        assert_eq!(config.dht_shared_key(), &TEST_KEY_A);
        assert_eq!(config.name(), "Test config");

        // Testing setters and getters
        config.set_dht_shared_key(TEST_KEY_B);
        config.set_name(String::from("New name"));
        assert_eq!(config.dht_shared_key(), &TEST_KEY_B);
        assert_eq!(config.name(), "New name");
    }

    #[test]
    fn test_device_info() {
        let mut expected_private_key_file = sifis_home_path();
        expected_private_key_file.push("private.pem");

        // Testing constructor and getters
        let mut device = DeviceInfo::new("Test Device".to_string()).unwrap();
        assert_eq!(device.authorization_key().len(), 32);
        assert_eq!(device.private_key_file(), &expected_private_key_file);
        assert_eq!(device.product_name(), "Test Device");
        assert_eq!(device.uuid().get_version_num(), 7);

        // Testing setters and getters
        let new_uuid = uuid!("123e4567-e89b-12d3-a456-426614174000");
        device.set_authorization_key(TEST_KEY_A);
        device.set_private_key_file(PathBuf::from("/tmp/private.key"));
        device.set_product_name("New name".to_string());
        device.set_uuid(new_uuid);
        assert_eq!(device.authorization_key(), &TEST_KEY_A);
        assert_eq!(
            device.private_key_file(),
            &PathBuf::from("/tmp/private.key")
        );
        assert_eq!(device.product_name(), "New name");
        assert_eq!(device.uuid(), &new_uuid);
    }
}
