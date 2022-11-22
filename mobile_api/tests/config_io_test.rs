use mobile_api::configs::{DeviceConfig, DeviceInfo};
use mobile_api::security::SecurityKey;
use mobile_api::SIFIS_HOME_PATH_ENV;
use std::env;
use tempfile::TempDir;

#[test]
#[cfg_attr(miri, ignore)] // File operations not available for miri when isolation is enabled
fn test_device_config_io() {
    // Create a temporary directory and use it as SIFIS_HOME_PATH
    let tmp_dir = TempDir::new().unwrap();
    env::set_var(SIFIS_HOME_PATH_ENV, tmp_dir.path());

    // Loading the config should cause error at first because file is not found
    let config_result = DeviceConfig::load();
    assert!(config_result.is_err());

    // Written and loaded configuration should be equal
    let save_config = DeviceConfig::new(SecurityKey::new().unwrap(), String::from("Test Device"));
    save_config.save().unwrap();
    let load_config = DeviceConfig::load().unwrap();
    assert_eq!(save_config, load_config);

    // Loading information should cause error at first because file is not found
    let info_result = DeviceInfo::load();
    assert!(info_result.is_err());

    // Written and loaded configuration should be equal
    let save_info = DeviceInfo::new(String::from("Test device")).unwrap();
    save_info.save().unwrap();
    let load_info = DeviceInfo::load().unwrap();
    assert_eq!(save_info, load_info);
}
