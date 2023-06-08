//! Endpoints for Running Commands
//!
//! These endpoints allow Mobile Application to give commands to the Smart Device,

use crate::api_common::{ApiKey, ApiKeyError, ErrorResponse, GenericResponse, OkResponse};
use crate::state::{BusyGuard, DeviceState};
use rocket::{get, State};
use rocket_okapi::openapi;
use std::path::PathBuf;
use std::process::Command;

/// # Reset the device back to factory settings
///
/// Calling this endpoint will delete any settings changes to the device. After this, we still need
/// to call the `/command/restart` endpoint to restart the device.
///
/// After the reboot, the device returns to the initialization phase, waiting for activation with
/// the mobile application.
///
/// To perform a factory reset, the `confirm` parameter must be set to the message
/// `I really want to perform a factory reset`.
#[openapi(tag = "Commands")]
#[get("/command/factory_reset?<confirm>")]
pub async fn factory_reset(
    key: Result<ApiKey, ApiKeyError>,
    state: &State<DeviceState>,
    confirm: Option<&str>,
) -> GenericResponse {
    match key {
        Ok(_) => match confirm {
            Some("I really want to perform a factory reset") => {
                match BusyGuard::try_busy(state, "A factory reset is performed.") {
                    Ok(_) => {
                        if let Err(err) = state.set_config(None) {
                            return GenericResponse::Error(ErrorResponse::internal_server_error(
                                err.to_string(),
                            ));
                        }
                        if let Err(err) = run_script(state, "factory_reset.sh") {
                            return GenericResponse::Error(ErrorResponse::internal_server_error(
                                err.to_string(),
                            ));
                        }
                        GenericResponse::Ok(OkResponse::message("Factory reset complete."))
                    }
                    Err(busy) => GenericResponse::Busy(ErrorResponse::service_unavailable(busy)),
                }
            }
            _ => GenericResponse::BadRequest(ErrorResponse::bad_request(Some(
                "The required confirm parameter was not correct or set.",
            ))),
        },
        Err(err) => match err {
            ApiKeyError::InvalidKey(content) => GenericResponse::BadRequest(content),
            ApiKeyError::WrongKey(content) => GenericResponse::Unauthorized(content),
        },
    }
}

/// # Restart the device
///
/// Calling this endpoint will initiate a device reboot.
#[openapi(tag = "Commands")]
#[get("/command/restart")]
pub async fn restart(
    key: Result<ApiKey, ApiKeyError>,
    state: &State<DeviceState>,
) -> GenericResponse {
    match key {
        Ok(_) => match BusyGuard::try_busy(state, "The device is restarting.") {
            Ok(_) => {
                if let Err(err) = run_script(state, "restart.sh") {
                    return GenericResponse::Error(ErrorResponse::internal_server_error(
                        err.to_string(),
                    ));
                }
                GenericResponse::Ok(OkResponse::message("System will now restart."))
            }
            Err(reason) => GenericResponse::Busy(ErrorResponse::service_unavailable(reason)),
        },
        Err(err) => match err {
            ApiKeyError::InvalidKey(content) => GenericResponse::BadRequest(content),
            ApiKeyError::WrongKey(content) => GenericResponse::Unauthorized(content),
        },
    }
}

/// # Shutdown the device
///
/// Calling this endpoint will initiate a shutdown of the device.
#[openapi(tag = "Commands")]
#[get("/command/shutdown")]
pub async fn shutdown(
    key: Result<ApiKey, ApiKeyError>,
    state: &State<DeviceState>,
) -> GenericResponse {
    match key {
        Ok(_) => match BusyGuard::try_busy(state, "The device is shutting down.") {
            Ok(_) => {
                if let Err(err) = run_script(state, "shutdown.sh") {
                    return GenericResponse::Error(ErrorResponse::internal_server_error(
                        err.to_string(),
                    ));
                }
                GenericResponse::Ok(OkResponse::message("System will now power off."))
            }
            Err(reason) => GenericResponse::Busy(ErrorResponse::service_unavailable(reason)),
        },
        Err(err) => match err {
            ApiKeyError::InvalidKey(content) => GenericResponse::BadRequest(content),
            ApiKeyError::WrongKey(content) => GenericResponse::Unauthorized(content),
        },
    }
}

/// Run script from the server `scripts` directory
fn run_script(
    state: &State<DeviceState>,
    script_name: &'static str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut script = match std::env::var("MOBILE_API_SCRIPTS_PATH") {
        Ok(path) => PathBuf::from(path),
        Err(_) => state.resource_path("scripts")?,
    };
    script.push(script_name);
    println!("Running: {:?}", script);
    let mut command = Command::new(script);
    let output = command.output()?;
    if output.status.success() {
        let output_stdout = String::from_utf8_lossy(&output.stdout);
        if !output_stdout.is_empty() {
            println!("{}", output_stdout)
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::api_common::{ErrorResponse, OkResponse};
    use crate::api_v1::tests_common::*;
    use rocket::fs::relative;
    use rocket::http::Status;
    use std::path::PathBuf;
    use std::time::Duration;

    // Test ignored for Miri because the server has time and io-related
    // functions that are not available in isolation mode
    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_factory_reset() {
        std::env::set_var("MOBILE_API_SCRIPTS_PATH", relative!("tests/scripts/"));
        let uri = "/v1/command/factory_reset";
        let (test_dir, client) = create_test_setup();
        test_invalid_auth_get(&client, uri);

        // Save test config
        let test_config = create_test_config();
        let mut test_config_file = PathBuf::from(test_dir.path());
        test_config_file.push("sifis-home");
        test_config_file.push("config.json");
        test_config.save_to(&test_config_file).unwrap();

        // Reset needs extra query parameter
        let response = client.get(uri).header(api_key_header()).dispatch();
        assert_eq!(response.status(), Status::BadRequest);
        let error_response = response.into_json::<ErrorResponse>().unwrap();
        assert_eq!(error_response.error.code, 400);
        assert!(test_config_file.exists());

        // Here we give the required extra parameter
        let (runtime, handle) = make_script_run_checker("FactoryReset", Duration::from_secs(10));
        let response = client
            .get("/v1/command/factory_reset?confirm=I%20really%20want%20to%20perform%20a%20factory%20reset")
            .header(api_key_header())
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let ok_response = response.into_json::<OkResponse>().unwrap();
        assert_eq!(ok_response.code, 200);
        assert!(!test_config_file.exists());
        let script = runtime.block_on(handle).unwrap().unwrap();
        assert_eq!(script, "factory_reset.sh");
    }

    // Test ignored for Miri because the server has time and io-related
    // functions that are not available in isolation mode
    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_restart() {
        std::env::set_var("MOBILE_API_SCRIPTS_PATH", relative!("tests/scripts/"));
        let uri = "/v1/command/restart";
        let (_test_dir, client) = create_test_setup();
        test_invalid_auth_get(&client, uri);

        let (runtime, handle) = make_script_run_checker("Restart", Duration::from_secs(10));
        let response = client.get(uri).header(api_key_header()).dispatch();
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
        let uri = "/v1/command/shutdown";
        let (_test_dir, client) = create_test_setup();
        test_invalid_auth_get(&client, uri);

        let (runtime, handle) = make_script_run_checker("Shutdown", Duration::from_secs(10));
        let response = client.get(uri).header(api_key_header()).dispatch();
        assert_eq!(response.status(), Status::Ok);

        let ok_response = response.into_json::<OkResponse>().unwrap();
        assert_eq!(ok_response.code, 200);
        assert_eq!(ok_response.message, "System will now power off.");

        let script = runtime.block_on(handle).unwrap().unwrap();
        assert_eq!(script, "shutdown.sh");
    }
}
