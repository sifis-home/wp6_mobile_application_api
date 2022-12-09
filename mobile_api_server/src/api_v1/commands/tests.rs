use crate::api_common::{ErrorResponse, OkResponse};
use crate::api_v1::tests_common::*;
use mobile_api::device_config_path;
use rocket::fs::relative;
use rocket::http::Status;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

// Test ignored for Miri because the server has time and io-related
// functions that are not available in isolation mode
#[cfg_attr(miri, ignore)]
#[test]
fn test_factory_reset() {
    // Using scripts from tests folder
    std::env::set_var("MOBILE_API_SCRIPTS_PATH", relative!("tests/scripts/"));

    // Making temporary directory for testing
    let tmp_dir = TempDir::new().unwrap();
    let mut tmp_sifis_home_path = PathBuf::from(tmp_dir.path());
    tmp_sifis_home_path.push("sifis-home");
    std::fs::create_dir_all(&tmp_sifis_home_path).unwrap();
    std::env::set_var("SIFIS_HOME_PATH", tmp_sifis_home_path.into_os_string());

    // Save test config
    let test_config = make_test_device_config();
    test_config.save().unwrap();

    // Reset needs extra query parameter
    let client = make_test_client();
    let response = client.get("/v1/command/factory_reset").dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    let error_response = response.into_json::<ErrorResponse>().unwrap();
    assert_eq!(error_response.error.code, 400);
    assert!(
        device_config_path().exists(),
        "{:?} should still exists",
        device_config_path()
    );

    // Here we give the required extra parameter
    let (runtime, handle) = make_script_run_checker("FactoryReset", Duration::from_secs(10));
    let response = client
        .get("/v1/command/factory_reset?confirm=I%20really%20want%20to%20perform%20a%20factory%20reset")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let ok_response = response.into_json::<OkResponse>().unwrap();
    assert_eq!(ok_response.code, 200);
    assert!(
        !device_config_path().exists(),
        "{:?} should no longer exists",
        device_config_path()
    );
    let script = runtime.block_on(handle).unwrap().unwrap();
    assert_eq!(script, "factory_reset.sh");
}

// Test ignored for Miri because the server has time and io-related
// functions that are not available in isolation mode
#[cfg_attr(miri, ignore)]
#[test]
fn test_restart() {
    std::env::set_var("MOBILE_API_SCRIPTS_PATH", relative!("tests/scripts/"));
    let (runtime, handle) = make_script_run_checker("Restart", Duration::from_secs(10));
    let client = make_test_client();
    let response = client.get("/v1/command/restart").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let ok_response = response.into_json::<OkResponse>().unwrap();
    assert_eq!(ok_response.code, 200);
    assert_eq!(ok_response.message, "System will now restart.");
    let script = runtime.block_on(handle).unwrap().unwrap();
    assert_eq!(script, "restart.sh");
}

// Test ignored for Miri because the server has time and io-related
// functions that are not available in isolation mode
#[cfg_attr(miri, ignore)]
#[test]
fn test_shutdown() {
    std::env::set_var("MOBILE_API_SCRIPTS_PATH", relative!("tests/scripts/"));
    let (runtime, handle) = make_script_run_checker("Shutdown", Duration::from_secs(10));
    let client = make_test_client();
    let response = client.get("/v1/command/shutdown").dispatch();
    let ok_response = response.into_json::<OkResponse>().unwrap();
    assert_eq!(ok_response.code, 200);
    assert_eq!(ok_response.message, "System will now power off.");
    let script = runtime.block_on(handle).unwrap().unwrap();
    assert_eq!(script, "shutdown.sh");
}
