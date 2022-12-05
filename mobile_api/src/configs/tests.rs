use super::*;
use uuid::uuid;

const TEST_KEY_A: SecurityKey = SecurityKey::from_bytes([
    0xf0, 0xe1, 0xd2, 0xc3, 0xb4, 0xa5, 0x96, 0x87, 0x78, 0x69, 0x5a, 0x4b, 0x3c, 0x2d, 0x1e, 0x0f,
    0x0f, 0x1e, 0x2d, 0x3c, 0x4b, 0x5a, 0x69, 0x78, 0x87, 0x96, 0xa5, 0xb4, 0xc3, 0xd2, 0xe1, 0xf0,
]);

const TEST_KEY_B: SecurityKey = SecurityKey::from_bytes([
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
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
    let mut expected_private_key_file = sifis_home_path();
    expected_private_key_file.push("private.pem");

    // Testing constructor and getters
    let mut device = DeviceInfo::new("Test Device".to_string()).unwrap();
    assert!(!device.authorization_key().is_null());
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

#[test]
fn test_device_info_serde() {
    // Testing human readable with JSON
    let info_a = DeviceInfo::new(String::from("Test device")).unwrap();
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
