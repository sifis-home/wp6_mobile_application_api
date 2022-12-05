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
use crate::security::{SecurityKey, SRNG};
use crate::{device_config_path, device_info_path, sifis_home_path};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[cfg(test)]
mod tests;

/// Smart Device Configuration
#[derive(Debug, Deserialize, Eq, PartialEq, JsonSchema, Serialize)]
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

    /// Load from the default location
    ///
    /// This Convenience function tries to load a configuration file from
    /// the location returned by the [device_config_path].
    pub fn load() -> Result<DeviceConfig> {
        Self::load_from(&device_config_path())
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

    /// Write config to the default location.
    ///
    /// This convenience function tries to write configuration
    /// to the file path given by the [device_config_path].
    pub fn save(&self) -> Result<()> {
        self.save_to(&device_config_path())
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
    /// Create a new device information
    ///
    /// Product name is required, other information is automatically generated.
    pub fn new(product_name: String) -> Result<DeviceInfo> {
        let srng = SRNG::new();
        let mut private_key_file = sifis_home_path();
        private_key_file.push("private.pem");
        Ok(DeviceInfo {
            authorization_key: srng.generate_key()?,
            private_key_file,
            product_name,
            uuid: srng.generate_uuid()?,
        })
    }

    /// Create a new device information from known values
    pub fn from(
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

    /// Load from the default location
    ///
    /// This Convenience function tries to load a information file from
    /// the location returned by the [device_info_path].
    pub fn load() -> Result<DeviceInfo> {
        Self::load_from(&device_info_path())
    }

    /// Load from file
    ///
    /// Tries to load and parse device information from the given *file* path.
    pub fn load_from(file: &Path) -> Result<DeviceInfo> {
        let info_json = fs::read_to_string(file)?;
        Ok(serde_json::from_str::<DeviceInfo>(&info_json)?)
    }

    /// Write info to the default location.
    ///
    /// This convenience function tries to write information
    /// to the file path given by the [device_info_path].
    pub fn save(&self) -> Result<()> {
        self.save_to(&device_info_path())
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
