use mobile_api::{device_config_path, device_info_path, sifis_home_path, SIFIS_HOME_PATH_ENV};
use std::env;

#[test]
fn test_paths() {
    // Testing default paths
    env::remove_var(SIFIS_HOME_PATH_ENV);
    let home_path = sifis_home_path();
    let config_path = device_config_path();
    let info_path = device_info_path();
    assert_eq!(home_path.to_str().unwrap(), "/opt/sifis-home/");
    assert_eq!(config_path.to_str().unwrap(), "/opt/sifis-home/config.json");
    assert_eq!(info_path.to_str().unwrap(), "/opt/sifis-home/device.json");

    // Testing with environment variable ending with '/'
    env::set_var(SIFIS_HOME_PATH_ENV, "/usr/lib/sifis-home/");
    let home_path = sifis_home_path();
    let config_path = device_config_path();
    let info_path = device_info_path();
    assert_eq!(home_path.to_str().unwrap(), "/usr/lib/sifis-home/");
    assert_eq!(
        config_path.to_str().unwrap(),
        "/usr/lib/sifis-home/config.json"
    );
    assert_eq!(
        info_path.to_str().unwrap(),
        "/usr/lib/sifis-home/device.json"
    );

    // Testing with environment variable not ending with '/'
    env::set_var(SIFIS_HOME_PATH_ENV, "/usr/lib/sifis-home");
    let home_path = sifis_home_path();
    let config_path = device_config_path();
    let info_path = device_info_path();
    assert_eq!(home_path.to_str().unwrap(), "/usr/lib/sifis-home");
    assert_eq!(
        config_path.to_str().unwrap(),
        "/usr/lib/sifis-home/config.json"
    );
    assert_eq!(
        info_path.to_str().unwrap(),
        "/usr/lib/sifis-home/device.json"
    );
}
