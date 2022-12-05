//! Endpoints for Running Commands
//!
//! These endpoints allow Mobile Application to give commands to the Smart Device,

use crate::state::{BusyGuard, DeviceState};
use rocket::{get, Responder, State};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::Responses;
use rocket_okapi::openapi;
use rocket_okapi::response::OpenApiResponderInner;
use rocket_okapi::util::add_schema_response;

#[cfg(test)]
mod tests;

/// Command HTTP responses
#[derive(Responder)]
pub enum CommandResponse {
    /// Server is busy with message in text/plain
    #[response(status = 503)]
    Busy(&'static str),
    /// Command was okay with message in text/plain
    #[response(status = 200)]
    TextOk(&'static str),
}

impl OpenApiResponderInner for CommandResponse {
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut responses = Responses::default();
        let schema = gen.json_schema::<String>();
        add_schema_response(&mut responses, 200, "text/plain", schema.clone())?;
        add_schema_response(&mut responses, 503, "text/plain", schema)?;
        Ok(responses)
    }
}

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
pub async fn factory_reset(state: &State<DeviceState>, confirm: Option<&str>) -> CommandResponse {
    match confirm {
        Some("I really want to perform a factory reset") => {
            match BusyGuard::try_busy(state, "A factory reset is performed") {
                Ok(_) => CommandResponse::TextOk("Factory reset is not implemented yet"),
                Err(reason) => CommandResponse::Busy(reason),
            }
        }
        _ => CommandResponse::TextOk(concat!(
            "To perform a factory reset, the `confirm` parameter must be set to the ",
            "message `I really want to perform a factory reset`"
        )),
    }
}

/// # Restart the device
///
/// Calling this endpoint will initiate a device reboot.
#[openapi(tag = "Commands")]
#[get("/command/restart")]
pub async fn restart(state: &State<DeviceState>) -> CommandResponse {
    match state.set_busy("The device is restarting") {
        Ok(_) => CommandResponse::TextOk("Not implemented yet"),
        Err(reason) => CommandResponse::Busy(reason),
    }
}

/// # Shutdown the device
///
/// Calling this endpoint will initiate a shutdown of the device.
#[openapi(tag = "Commands")]
#[get("/command/shutdown")]
pub async fn shutdown(state: &State<DeviceState>) -> CommandResponse {
    match state.set_busy("The device is shutting down") {
        Ok(_) => CommandResponse::TextOk("Not implemented yet"),
        Err(reason) => CommandResponse::Busy(reason),
    }
}
