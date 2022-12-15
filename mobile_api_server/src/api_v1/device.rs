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

/// Status Endpoint Response
#[derive(Responder)]
pub enum StatusResponse {
    /// Status is always available and returns status information with 200 OK response.
    #[response(status = 200, content_type = "json")]
    Ok(Json<DeviceStatus>),
}

impl OpenApiResponderInner for StatusResponse {
    /// Generating responses for the status endpoint
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

/// Possible responses for the configuration GET endpoint
#[derive(Responder)]
pub enum GetConfigResponse {
    /// 200 OK, configuration is available
    #[response(status = 200, content_type = "json")]
    Ok(Json<DeviceConfig>),

    /// 404 Not Found, configuration is not done
    #[response(status = 404, content_type = "json")]
    NotFound(Json<ErrorResponse>),
}

impl OpenApiResponderInner for GetConfigResponse {
    /// Generating responses for the configuration GET endpoint
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

#[cfg(test)]
mod tests {
    use crate::api_v1::tests_common::{create_test_config, create_test_setup};
    use crate::device_status::DeviceStatus;
    use mobile_api::configs::DeviceConfig;
    use rocket::http::{ContentType, Status};

    // Test ignored for Miri because the server has time and io-related
    // functions that are not available in isolation mode
    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_status() {
        let (_test_dir, client) = create_test_setup();

        let response = client.get("/v1/device/status").dispatch();
        assert_eq!(response.status(), Status::Ok);

        let device_status = response.into_json::<DeviceStatus>();
        assert!(device_status.is_some());
    }

    // Test ignored for Miri because the server has time and io-related
    // functions that are not available in isolation mode
    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_configuration() {
        let uri = "/v1/device/configuration";

        // Should not have config yet
        let (_test_dir, client) = create_test_setup();
        let response = client.get(uri).dispatch();
        assert_eq!(response.status(), Status::NotFound);

        // Sending test configuration
        let test_config = create_test_config();
        let test_config_json = serde_json::to_string(&test_config).unwrap();
        let response = client
            .put(uri)
            .header(ContentType::JSON)
            .body(test_config_json)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);

        // Should have the same config now
        let response = client.get(uri).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let config = response.into_json::<DeviceConfig>().unwrap();
        assert_eq!(config, test_config);
    }
}
