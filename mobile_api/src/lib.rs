//! Smart Device Mobile API
//!
//! This crate provides the functionality required for the Smart Device Mobile API service but can
//! be helpful for other SIFIS-Home services.

use crate::configs::{DeviceConfig, DeviceInfo};
use crate::error::Result;
use crate::security::SRNG;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::{env, fs};

pub mod configs;
pub mod error;
pub mod security;

/// Environment variable name for SIFIS-Home configuration files path
pub const SIFIS_HOME_PATH_ENV: &str = "SIFIS_HOME_PATH";

/// SIFIS Home instance
///
/// The instance knows the location of the configuration
/// files and shares [SRNG] for generating secure keys.
pub struct SifisHome {
    /// The path where the SIFIS-Home files are placed
    sifis_home_path: PathBuf,

    /// Shared Secure Random Number Generator
    srng: SRNG,
}

impl SifisHome {
    /// Creates default instance
    ///
    /// Creates instance that uses default home path that is either `/opt/sifis-home/`
    /// or path given with the `SIFIS_HOME_PATH` environment variable.
    pub fn new() -> SifisHome {
        Self::new_with_path(PathBuf::from(match env::var(SIFIS_HOME_PATH_ENV) {
            Ok(path) => path,
            Err(_) => String::from("/opt/sifis-home/"),
        }))
    }

    /// Create instance using a custom path
    pub fn new_with_path(sifis_home_path: PathBuf) -> SifisHome {
        SifisHome {
            sifis_home_path,
            srng: SRNG::new(),
        }
    }

    /// Path to configuration files
    pub fn home_path(&self) -> &Path {
        &self.sifis_home_path
    }

    /// Path to device configuration file `config.json`
    pub fn config_file_path(&self) -> PathBuf {
        let mut path = self.sifis_home_path.clone();
        path.push("config.json");
        path
    }

    /// Path to device information file `device.json`
    pub fn info_file_path(&self) -> PathBuf {
        let mut path = self.sifis_home_path.clone();
        path.push("device.json");
        path
    }

    /// Create a new device information
    ///
    /// Product name is required, other information is automatically generated.
    pub fn new_info(&self, product_name: String) -> Result<DeviceInfo> {
        let mut private_key_file = self.sifis_home_path.clone();
        private_key_file.push("private.pem");
        Ok(DeviceInfo::new(
            product_name,
            self.srng.generate_key()?,
            private_key_file,
            self.srng.generate_uuid()?,
        ))
    }

    /// Load device info from the default location
    ///
    /// This Convenience function tries to load a information file from
    /// the location returned by the [device_info_path].
    pub fn load_info(&self) -> Result<DeviceInfo> {
        DeviceInfo::load_from(&self.info_file_path())
    }

    /// Write device info to the default location.
    ///
    /// This convenience function tries to write information
    /// to the file path given by the [device_info_path].
    pub fn save_info(&self, device_info: &DeviceInfo) -> Result<()> {
        device_info.save_to(&self.info_file_path())
    }

    /// Load device configuration from default location
    ///
    /// This Convenience function tries to load a configuration file from
    /// the location returned by the [device_config_path].
    pub fn load_config(&self) -> Result<DeviceConfig> {
        DeviceConfig::load_from(&self.config_file_path())
    }

    /// Removes configuration file `config.json`
    ///
    /// Returns Ok if file is removed or does not exists. Otherwise error is returned.
    pub fn remove_config(&self) -> Result<()> {
        match fs::remove_file(self.config_file_path()) {
            Ok(_) => Ok(()),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => Ok(()), // This is acceptable
                _ => Err(err.into()),
            },
        }
    }

    /// Write config to the default location.
    ///
    /// This convenience function tries to write configuration
    /// to the file path given by the [device_config_path].
    pub fn save_config(&self, config: &DeviceConfig) -> Result<()> {
        config.save_to(&self.config_file_path())
    }
}

impl Default for SifisHome {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::SecurityKey;
    use tempfile::TempDir;

    #[test]
    pub fn test_sifis_home_new() {
        // This is the only unit tests that should set SIFIS_HOME_PATH environment variable!
        env::set_var(SIFIS_HOME_PATH_ENV, "/test/sifis-home/");
        let sifis_home = SifisHome::new();
        assert_eq!(sifis_home.home_path(), Path::new("/test/sifis-home"));
        assert_eq!(
            sifis_home.info_file_path(),
            Path::new("/test/sifis-home/device.json")
        );
        assert_eq!(
            sifis_home.config_file_path(),
            Path::new("/test/sifis-home/config.json")
        );
    }

    #[test]
    pub fn test_sifis_home_new_with_path() {
        let sifis_home = SifisHome::new_with_path(PathBuf::from("/tmp/sifis-home"));
        assert_eq!(sifis_home.home_path(), Path::new("/tmp/sifis-home"));
        assert_eq!(
            sifis_home.info_file_path(),
            Path::new("/tmp/sifis-home/device.json")
        );
        assert_eq!(
            sifis_home.config_file_path(),
            Path::new("/tmp/sifis-home/config.json")
        );
    }

    #[cfg_attr(miri, ignore)] // File operations are not available with miri
    #[test]
    pub fn test_remove_config() {
        let test_dir = TempDir::new().unwrap();
        let sifis_home = SifisHome::new_with_path(PathBuf::from(test_dir.path()));
        let test_key = SecurityKey::from_bytes([
            0xf0, 0xe1, 0xd2, 0xc3, 0xb4, 0xa5, 0x96, 0x87, 0x78, 0x69, 0x5a, 0x4b, 0x3c, 0x2d,
            0x1e, 0x0f, 0x0f, 0x1e, 0x2d, 0x3c, 0x4b, 0x5a, 0x69, 0x78, 0x87, 0x96, 0xa5, 0xb4,
            0xc3, 0xd2, 0xe1, 0xf0,
        ]);
        let test_config = DeviceConfig::new(test_key, "Test".to_string());
        sifis_home.save_config(&test_config).unwrap();

        assert!(sifis_home.config_file_path().exists());
        assert!(sifis_home.remove_config().is_ok());
        assert!(!sifis_home.config_file_path().exists());
        assert!(sifis_home.remove_config().is_ok()); // Should be okay even when config file is missing
    }
}
