//! Smart Device Mobile API
//!
//! This crate provides the functionality required for the Smart Device Mobile API service but can
//! be helpful for other SIFIS-Home services.

use std::env;
use std::path::PathBuf;

pub mod configs;
pub mod error;
pub mod security;

/// Environment variable name for SIFIS-Home configuration files path
pub const SIFIS_HOME_PATH_ENV: &str = "SIFIS_HOME_PATH";

/// Path to device configuration file
///
/// Convenience function to construct path of `sifis_home_path() + "config.json"`
pub fn device_config_path() -> PathBuf {
    let mut path = sifis_home_path();
    path.push("config.json");
    path
}

/// Path to device information file
///
/// Convenience function to construct path of `sifis_home_path() + "device.json"`
pub fn device_info_path() -> PathBuf {
    let mut path = sifis_home_path();
    path.push("device.json");
    path
}

/// Path to SIFIS-home configuration files
///
/// The path is made from the SIFIS_HOME_PATH environment variable when it is available.
/// Otherwise, the function returns `/opt/sifis-home/` as the default path.
pub fn sifis_home_path() -> PathBuf {
    PathBuf::from(match env::var(SIFIS_HOME_PATH_ENV) {
        Ok(path) => path,
        Err(_) => String::from("/opt/sifis-home/"),
    })
}
