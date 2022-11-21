//! Endpoints for Running Commands
//!
//! These endpoints allow Mobile Application to give commands to the Smart Device,

use rocket::get;
use rocket_okapi::openapi;

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
pub async fn factory_reset(confirm: Option<&str>) -> &'static str {
    match confirm {
        Some("I really want to perform a factory reset") => "Factory reset is not implemented yet",
        _ => {
            concat!(
                "To perform a factory reset, the `confirm` parameter must be set to the ",
                "message `I really want to perform a factory reset`"
            )
        }
    }
}

/// # Restart the device
///
/// Calling this endpoint will initiate a device reboot.
#[openapi(tag = "Commands")]
#[get("/command/restart")]
pub async fn restart() -> &'static str {
    "Not implemented yet"
}

/// # Shutdown the device
///
/// Calling this endpoint will initiate a shutdown of the device.
#[openapi(tag = "Commands")]
#[get("/command/shutdown")]
pub async fn shutdown() -> &'static str {
    "Not implemented yet"
}
