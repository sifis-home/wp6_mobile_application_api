//! Endpoints for Device Information and Configuration
//!
//! These endpoints allow Mobile Application to check device status, read and set configuration.

use crate::api_common::*;
use crate::device_status::DeviceStatus;
use crate::state::{BusyGuard, DeviceState};
use mobile_api::configs::DeviceConfig;
use rocket::serde::json::Json;
use rocket::{get, put, Responder, State};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::Responses;
use rocket_okapi::openapi;
use rocket_okapi::response::OpenApiResponderInner;

#[cfg(test)]
mod tests;

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
pub async fn status(state: &State<DeviceState>) -> StatusResponse {
    StatusResponse::Ok(Json(state.device_status()))
}

#[derive(Responder)]
pub enum StatusResponse {
    #[response(status = 200, content_type = "json")]
    Ok(Json<DeviceStatus>),
}

impl OpenApiResponderInner for StatusResponse {
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        make_json_responses(vec![(200, gen.json_schema::<DeviceStatus>(), None)])
    }
}

/// # Device configuration
///
/// Returns the device settings or 404 if the configuration is not done yet.
/// Use PUT /device/configuration to set the configuration.
#[openapi(tag = "Device")]
#[get("/device/configuration")]
pub async fn get_config(state: &State<DeviceState>) -> GetConfigResponse {
    match state.get_config() {
        None => GetConfigResponse::NotFound(ErrorResponse::not_found(Some(
            "This device has not been configured yet.",
        ))),
        Some(config) => GetConfigResponse::Ok(Json(config)),
    }
}

#[derive(Responder)]
pub enum GetConfigResponse {
    #[response(status = 200, content_type = "json")]
    Ok(Json<DeviceConfig>),

    #[response(status = 404, content_type = "json")]
    NotFound(Json<ErrorResponse>),
}

impl OpenApiResponderInner for GetConfigResponse {
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        make_json_responses(vec![
            (200, gen.json_schema::<DeviceConfig>(), None),
            (
                404,
                gen.json_schema::<ErrorResponse>(),
                Some("This device has not been configured yet."),
            ),
        ])
    }
}

/// # Set device configuration
///
/// The device settings are sent in JSON format in the body of the message. After this, the device
/// must be restarted using the `/commands/restart` endpoint.
#[openapi(tag = "Device")]
#[put("/device/configuration", data = "<config>")]
pub async fn set_config(
    state: &State<DeviceState>,
    config: Json<DeviceConfig>,
) -> OkErrorBusyResponse {
    match BusyGuard::try_busy(state, "Saving device configuration.") {
        Ok(_) => match state.set_config(Some(config.0)) {
            Ok(_) => OkErrorBusyResponse::Ok(OkResponse::message("Configuration saved.")),
            Err(error) => {
                OkErrorBusyResponse::Error(ErrorResponse::internal_server_error(error.to_string()))
            }
        },
        Err(busy) => OkErrorBusyResponse::Busy(ErrorResponse::service_unavailable(busy)),
    }
}
