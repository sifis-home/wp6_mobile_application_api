use crate::api_v1::tests_common::{make_test_client, TEST_SHARED_DHT_KEY};
use crate::device_status::DeviceStatus;
use mobile_api::configs::DeviceConfig;
use rocket::http::{ContentType, Status};

// Test ignored for Miri because the server has time and io-related
// functions that are not available in isolation mode
#[cfg_attr(miri, ignore)]
#[test]
fn test_status() {
    let client = make_test_client();
    let response = client.get("/v1/device/status").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let device_status = response.into_json::<DeviceStatus>();
    assert!(device_status.is_some());
}

// Test ignored for Miri because the server has time and io-related
// functions that are not available in isolation mode
#[cfg_attr(miri, ignore)]
#[test]
#[should_panic] // Not implemented
fn test_configuration() {
    let uri = "/v1/device/configuration";

    // Should not have config yet
    let client = make_test_client();
    let response = client.get(uri).dispatch();
    assert_eq!(response.status(), Status::NotFound);

    // Sending test configuration
    let test_config = DeviceConfig::new(TEST_SHARED_DHT_KEY, "Test Config".to_string());
    let test_config_json = serde_json::to_string(&test_config).unwrap();
    let response = client
        .put(uri)
        .header(ContentType::JSON)
        .body(test_config_json)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    // Should have the same config now
    let client = make_test_client();
    let response = client.get(uri).dispatch();
    assert_eq!(response.status(), Status::Ok);
    let config = response.into_json::<DeviceConfig>().unwrap();
    assert_eq!(config, test_config);
}
