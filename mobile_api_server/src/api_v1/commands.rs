//! Endpoints for Running Commands
//!
//! These endpoints allow Mobile Application to give commands to the Smart Device,

use crate::api_common::{make_json_responses, ErrorResponse, OkErrorBusyResponse, OkResponse};
use crate::state::{BusyGuard, DeviceState};
use rocket::serde::json::Json;
use rocket::{get, Responder, Shutdown, State};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::Responses;
use rocket_okapi::openapi;
use rocket_okapi::response::OpenApiResponderInner;
use std::path::PathBuf;
use std::process::Command;

#[cfg(test)]
mod tests;

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
    state: &State<DeviceState>,
    confirm: Option<&str>,
) -> FactoryResetResponse {
    match confirm {
        Some("I really want to perform a factory reset") => {
            match BusyGuard::try_busy(state, "A factory reset is performed.") {
                Ok(_) => {
                    if let Err(err) = state.set_config(None) {
                        return FactoryResetResponse::Error(ErrorResponse::internal_server_error(
                            err.to_string(),
                        ));
                    }
                    if let Err(err) = run_script("factory_reset.sh") {
                        return FactoryResetResponse::Error(ErrorResponse::internal_server_error(
                            err.to_string(),
                        ));
                    }
                    FactoryResetResponse::Ok(OkResponse::message("Factory reset complete."))
                }
                Err(busy) => FactoryResetResponse::Busy(ErrorResponse::service_unavailable(busy)),
            }
        }
        _ => FactoryResetResponse::BadRequest(ErrorResponse::bad_request(Some(
            "The required confirm parameter was not correct or set.",
        ))),
    }
}

#[derive(Responder)]
pub enum FactoryResetResponse {
    #[response(status = 200, content_type = "json")]
    Ok(Json<OkResponse>),

    #[response(status = 400, content_type = "json")]
    BadRequest(Json<ErrorResponse>),

    #[response(status = 500, content_type = "json")]
    Error(Json<ErrorResponse>),

    #[response(status = 503, content_type = "json")]
    Busy(Json<ErrorResponse>),
}

impl OpenApiResponderInner for FactoryResetResponse {
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        make_json_responses(vec![
            (200, gen.json_schema::<OkResponse>(), None),
            (400, gen.json_schema::<ErrorResponse>(), None),
            (500, gen.json_schema::<ErrorResponse>(), None),
            (503, gen.json_schema::<ErrorResponse>(), None),
        ])
    }
}

/// # Restart the device
///
/// Calling this endpoint will initiate a device reboot.
#[openapi(tag = "Commands")]
#[get("/command/restart")]
pub async fn restart(state: &State<DeviceState>, shutdown: Shutdown) -> OkErrorBusyResponse {
    match BusyGuard::try_busy(state, "The device is restarting.") {
        Ok(_) => {
            if let Err(err) = run_script("restart.sh") {
                return OkErrorBusyResponse::Error(ErrorResponse::internal_server_error(
                    err.to_string(),
                ));
            }
            shutdown.notify();
            OkErrorBusyResponse::Ok(OkResponse::message("System will now restart."))
        }
        Err(reason) => OkErrorBusyResponse::Busy(ErrorResponse::service_unavailable(reason)),
    }
}

/// # Shutdown the device
///
/// Calling this endpoint will initiate a shutdown of the device.
#[openapi(tag = "Commands")]
#[get("/command/shutdown")]
pub async fn shutdown(state: &State<DeviceState>, shutdown: Shutdown) -> OkErrorBusyResponse {
    match BusyGuard::try_busy(state, "The device is shutting down.") {
        Ok(_) => {
            if let Err(err) = run_script("shutdown.sh") {
                return OkErrorBusyResponse::Error(ErrorResponse::internal_server_error(
                    err.to_string(),
                ));
            }
            shutdown.notify();
            OkErrorBusyResponse::Ok(OkResponse::message("System will now power off."))
        }
        Err(reason) => OkErrorBusyResponse::Busy(ErrorResponse::service_unavailable(reason)),
    }
}

/// Run script from the server `scripts` directory
fn run_script(script_name: &'static str) -> Result<(), Box<dyn std::error::Error>> {
    let mut script = match std::env::var("MOBILE_API_SCRIPTS_PATH") {
        Ok(path) => PathBuf::from(path),
        Err(_) => PathBuf::from(rocket::fs::relative!("scripts")),
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
