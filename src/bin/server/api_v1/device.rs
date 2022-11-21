//! Endpoints for Device Information and Configuration
//!
//! These endpoints allow Mobile Application to check device status, read and set configuration.

use mobile_api::configs::DeviceConfig;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, put};
use rocket_okapi::openapi;

/// # Device status
///
/// This endpoint provides information about the status of the device, such as:
///
/// * CPU usage
///
/// * Memory usage
///
/// * Disk space usage
///
/// * Uptime
///
/// * Load average
///
#[openapi(tag = "Device")]
#[get("/device/status")]
pub async fn status() -> &'static str {
    "Not implemented yet"
}

/// # Device configuration
///
/// Returns the device settings or 404 if the configuration is not done yet.
/// Use PUT /device/configuration to set the configuration.
#[openapi(tag = "Device")]
#[get("/device/configuration")]
pub async fn get_config() -> Result<Json<DeviceConfig>, Status> {
    Err(Status::NotFound)
}

/// # Set device configuration
///
/// The device settings are sent in JSON format in the body of the message. After this, the device
/// must be restarted using the `/commands/restart` endpoint.
#[openapi(tag = "Device")]
#[put("/device/configuration", data = "<config>")]
pub async fn set_config(config: Json<DeviceConfig>) -> &'static str {
    println!("Got configuration: {:#?}", config);
    "Not implemented yet"
}
