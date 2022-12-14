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
use crate::security::SecurityKey;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Smart Device Configuration
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, JsonSchema, Serialize)]
pub struct DeviceConfig {
    /// User-defined name for the Smart Device
    name: String,
    /// Shared key for DHT communication, 32 bytes in hex format
    dht_shared_key: SecurityKey,
}

impl DeviceConfig {
    /// Create a new configuration
    pub fn new(dht_shared_key: SecurityKey, name: String) -> DeviceConfig {
        DeviceConfig {
            dht_shared_key,
            name,
        }
    }

    /// Borrow shared DHT key
    pub fn dht_shared_key(&self) -> &SecurityKey {
        &self.dht_shared_key
    }

    /// Load from file
    ///
    /// Tries to load and parse configuration from the given *file* path.
    pub fn load_from(file: &Path) -> Result<DeviceConfig> {
        let config_json = fs::read_to_string(file)?;
        Ok(serde_json::from_str::<DeviceConfig>(&config_json)?)
    }

    /// Borrow device name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Save to file
    ///
    /// Tries to write configuration to the given *file* as pretty JSON.
    pub fn save_to(&self, file: &Path) -> Result<()> {
        let config_json = serde_json::to_string_pretty(&self)?;
        fs::write(file, config_json.as_bytes())?;
        Ok(())
    }

    /// Change shared DHT key
    pub fn set_dht_shared_key(&mut self, dht_shared_key: SecurityKey) {
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
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DeviceInfo {
    /// Product name
    product_name: String,
    /// 256-bit authorization key in hex format. SIFIS-Home mobile application needs this key to
    /// access configuration endpoints of the Smart Device Mobile API service.
    authorization_key: SecurityKey,
    /// Path to DHT private key file. The sifis-dht generates key file on the first run
    private_key_file: PathBuf,
    /// 128-bit UUID in standard hex format
    uuid: Uuid,
}

impl DeviceInfo {
    /// Create a new device information from known values
    pub fn new(
        product_name: String,
        authorization_key: SecurityKey,
        private_key_file: PathBuf,
        uuid: Uuid,
    ) -> DeviceInfo {
        DeviceInfo {
            product_name,
            authorization_key,
            private_key_file,
            uuid,
        }
    }

    /// Borrow authorization key
    pub fn authorization_key(&self) -> &SecurityKey {
        &self.authorization_key
    }

    /// Load from file
    ///
    /// Tries to load and parse device information from the given *file* path.
    pub fn load_from(file: &Path) -> Result<DeviceInfo> {
        let info_json = fs::read_to_string(file)?;
        Ok(serde_json::from_str::<DeviceInfo>(&info_json)?)
    }

    /// Save to file
    ///
    /// Tries to write device information to the given *file* as pretty JSON.
    pub fn save_to(&self, file: &Path) -> Result<()> {
        fs::write(file, self.to_json(true)?.as_bytes())?;
        Ok(())
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
    pub fn set_authorization_key(&mut self, authorization_key: SecurityKey) {
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

    /// Convenience function to turn device information to JSON
    pub fn to_json(&self, pretty: bool) -> Result<String> {
        Ok(match pretty {
            true => serde_json::to_string_pretty(&self)?,
            false => serde_json::to_string(&self)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SifisHome;
    use uuid::uuid;

    const TEST_KEY_A: SecurityKey = SecurityKey::from_bytes([
        0xf0, 0xe1, 0xd2, 0xc3, 0xb4, 0xa5, 0x96, 0x87, 0x78, 0x69, 0x5a, 0x4b, 0x3c, 0x2d, 0x1e,
        0x0f, 0x0f, 0x1e, 0x2d, 0x3c, 0x4b, 0x5a, 0x69, 0x78, 0x87, 0x96, 0xa5, 0xb4, 0xc3, 0xd2,
        0xe1, 0xf0,
    ]);

    const TEST_KEY_B: SecurityKey = SecurityKey::from_bytes([
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
        0x1e, 0x1f,
    ]);

    pub const TEST_UUID: Uuid = Uuid::from_bytes([
        0x12, 0x3e, 0x45, 0x67, 0xe8, 0x9b, 0x12, 0xd3, 0xa4, 0x56, 0x42, 0x66, 0x14, 0x17, 0x40,
        0x00,
    ]);

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
    fn test_device_config_serde() {
        // Testing human readable with JSON
        let config_a = DeviceConfig::new(SecurityKey::new().unwrap(), String::from("Test device"));
        let json = serde_json::to_string(&config_a).unwrap();
        let config_b = serde_json::from_str::<DeviceConfig>(&json).unwrap();
        assert_eq!(config_a, config_b);

        // Testing binary with MessagePack
        let buf = rmp_serde::to_vec(&config_a).unwrap();
        let config_b = rmp_serde::from_slice::<DeviceConfig>(&buf).unwrap();
        assert_eq!(config_a, config_b);
    }

    #[test]
    fn test_device_info() {
        // Testing construction with SifisHome
        let sifis_home = SifisHome::new();
        let mut expected_private_key_file = PathBuf::from(sifis_home.home_path());
        expected_private_key_file.push("private.pem");
        let device = sifis_home.new_info("Test Device".to_string()).unwrap();
        assert!(!device.authorization_key().is_null());
        assert_eq!(device.private_key_file(), &expected_private_key_file);
        assert_eq!(device.product_name(), "Test Device");
        assert_eq!(device.uuid().get_version_num(), 7);

        // Testing constructor and getters
        let test_private_key = "/tmp/test/private.key";
        let mut device = DeviceInfo::new(
            "Test Device".to_string(),
            TEST_KEY_A,
            PathBuf::from(test_private_key),
            TEST_UUID,
        );
        assert_eq!(device.authorization_key(), &TEST_KEY_A);
        assert_eq!(device.private_key_file(), Path::new(test_private_key));
        assert_eq!(device.product_name(), "Test Device");
        assert_eq!(device.uuid(), &TEST_UUID);

        // Testing setters and getters
        let new_uuid = uuid!("5f8b3c30-ec2f-4228-af3b-dde564985e60");
        let new_private_key = "/tmp/test/private.pem";
        device.set_authorization_key(TEST_KEY_B);
        device.set_private_key_file(PathBuf::from(&new_private_key));
        device.set_product_name("New name".to_string());
        device.set_uuid(new_uuid);
        assert_eq!(device.authorization_key(), &TEST_KEY_B);
        assert_eq!(device.private_key_file(), Path::new(new_private_key));
        assert_eq!(device.product_name(), "New name");
        assert_eq!(device.uuid(), &new_uuid);
    }

    #[test]
    fn test_device_info_serde() {
        let sifis_home = SifisHome::new();

        // Testing human readable with JSON
        let info_a = sifis_home.new_info(String::from("Test device")).unwrap();
        let json = serde_json::to_string(&info_a).unwrap();
        let info_b = serde_json::from_str::<DeviceInfo>(&json).unwrap();
        assert_eq!(info_a, info_b);

        // Testing binary with MessagePack
        let buf = rmp_serde::to_vec(&info_a).unwrap();
        let config_b = rmp_serde::from_slice::<DeviceInfo>(&buf).unwrap();
        assert_eq!(info_a, config_b);

        // Both compact and pretty JSON should result identical DeviceInfo object
        let compact_json = info_a.to_json(false).unwrap();
        let pretty_json = info_a.to_json(true).unwrap();
        assert_ne!(compact_json, pretty_json);
        let info_b = serde_json::from_str::<DeviceInfo>(&compact_json).unwrap();
        let info_c = serde_json::from_str::<DeviceInfo>(&pretty_json).unwrap();
        assert_eq!(info_a, info_b);
        assert_eq!(info_b, info_c);
        assert_eq!(info_b, info_c);
    }
}
