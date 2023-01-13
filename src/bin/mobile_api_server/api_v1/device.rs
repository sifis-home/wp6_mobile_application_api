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
pub async fn status(
    key: Result<ApiKey, ApiKeyError>,
    state: &State<DeviceState>,
) -> StatusResponse {
    match key {
        Ok(_) => StatusResponse::Ok(Json(state.device_status())),
        Err(err) => match err {
            ApiKeyError::InvalidKey(content) => StatusResponse::BadRequest(content),
            ApiKeyError::WrongKey(content) => StatusResponse::Unauthorized(content),
        },
    }
}

/// Status Endpoint Response
#[derive(Responder)]
pub enum StatusResponse {
    /// 200 OK
    #[response(status = 200, content_type = "json")]
    Ok(Json<DeviceStatus>),

    /// 400 Bad Request
    #[response(status = 400, content_type = "json")]
    BadRequest(Json<ErrorResponse>),

    /// 401 Unauthorized
    #[response(status = 401, content_type = "json")]
    Unauthorized(Json<ErrorResponse>),
}

impl OpenApiResponderInner for StatusResponse {
    /// Generating responses for the status endpoint
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        make_json_responses(vec![
            (200, gen.json_schema::<DeviceStatus>(), None),
            (400, gen.json_schema::<ErrorResponse>(), None),
            (401, gen.json_schema::<ErrorResponse>(), None),
        ])
    }
}

/// # Device configuration
///
/// Returns the device settings or 404 if the configuration is not done yet.
/// Use PUT /device/configuration to set the configuration.
#[openapi(tag = "Device")]
#[get("/device/configuration")]
pub async fn get_config(
    key: Result<ApiKey, ApiKeyError>,
    state: &State<DeviceState>,
) -> GetConfigResponse {
    match key {
        Ok(_) => match state.get_config() {
            None => GetConfigResponse::NotFound(ErrorResponse::not_found(Some(
                "This device has not been configured yet.",
            ))),
            Some(config) => GetConfigResponse::Ok(Json(config)),
        },
        Err(err) => match err {
            ApiKeyError::InvalidKey(content) => GetConfigResponse::BadRequest(content),
            ApiKeyError::WrongKey(content) => GetConfigResponse::Unauthorized(content),
        },
    }
}

/// Possible responses for the configuration GET endpoint
#[derive(Responder)]
pub enum GetConfigResponse {
    /// 200 OK, configuration is available
    #[response(status = 200, content_type = "json")]
    Ok(Json<DeviceConfig>),

    /// 400 Bad Request
    #[response(status = 400, content_type = "json")]
    BadRequest(Json<ErrorResponse>),

    /// 401 Unauthorized
    #[response(status = 401, content_type = "json")]
    Unauthorized(Json<ErrorResponse>),

    /// 404 Not Found, configuration is not done
    #[response(status = 404, content_type = "json")]
    NotFound(Json<ErrorResponse>),
}

impl OpenApiResponderInner for GetConfigResponse {
    /// Generating responses for the configuration GET endpoint
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        make_json_responses(vec![
            (200, gen.json_schema::<DeviceConfig>(), None),
            (400, gen.json_schema::<ErrorResponse>(), None),
            (401, gen.json_schema::<ErrorResponse>(), None),
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
    key: Result<ApiKey, ApiKeyError>,
    state: &State<DeviceState>,
    config: Json<DeviceConfig>,
) -> GenericResponse {
    match key {
        Ok(_) => match BusyGuard::try_busy(state, "Saving device configuration.") {
            Ok(_) => match state.set_config(Some(config.0)) {
                Ok(_) => GenericResponse::Ok(OkResponse::message("Configuration saved.")),
                Err(error) => {
                    GenericResponse::Error(ErrorResponse::internal_server_error(error.to_string()))
                }
            },
            Err(busy) => GenericResponse::Busy(ErrorResponse::service_unavailable(busy)),
        },
        Err(err) => match err {
            ApiKeyError::InvalidKey(content) => GenericResponse::BadRequest(content),
            ApiKeyError::WrongKey(content) => GenericResponse::Unauthorized(content),
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::api_common::ErrorResponse;
    use crate::api_v1::tests_common::{
        api_key_header, create_test_config, create_test_setup, test_invalid_auth_get,
    };
    use crate::device_status::DeviceStatus;
    use mobile_api::configs::DeviceConfig;
    use rocket::http::{ContentType, Header, Status};
    use rocket::local::blocking::Client;

    // Test ignored for Miri because the server has time and io-related
    // functions that are not available in isolation mode
    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_status() {
        let uri = "/v1/device/status";
        let (_test_dir, client) = create_test_setup();
        test_invalid_auth_get(&client, uri);

        let response = client.get(uri).header(api_key_header()).dispatch();
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
        let (_test_dir, client) = create_test_setup();
        test_invalid_auth_get(&client, uri);

        // We need to test PUT method for invalid authentication too
        let test_config = create_test_config();
        let test_config_json = serde_json::to_string(&test_config).unwrap();
        test_invalid_auth_put(&client, uri, &test_config_json);

        // Should not have config yet
        let response = client.get(uri).header(api_key_header()).dispatch();
        assert_eq!(response.status(), Status::NotFound);

        // Sending test configuration
        let response = client
            .put(uri)
            .header(api_key_header())
            .header(ContentType::JSON)
            .body(test_config_json)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);

        // Should have the same config now
        let response = client.get(uri).header(api_key_header()).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let config = response.into_json::<DeviceConfig>().unwrap();
        assert_eq!(config, test_config);
    }

    fn test_invalid_auth_put(client: &Client, uri: &str, body: &str) {
        // Testing request without api key
        let response = client.put(uri).body(body).dispatch();
        assert_eq!(response.status(), Status::BadRequest);
        let error_response = response.into_json::<ErrorResponse>().unwrap();
        assert_eq!(error_response.error.code, 400);
        assert_eq!(error_response.error.reason, "Bad Request");
        assert_eq!(
            error_response.error.description,
            "Missing `x-api-key` header."
        );

        // Testing request with invalid api key
        let response = client
            .put(uri)
            .header(Header::new("x-api-key", "invalid key"))
            .body(body)
            .dispatch();
        assert_eq!(response.status(), Status::BadRequest);
        let error_response = response.into_json::<ErrorResponse>().unwrap();
        assert_eq!(error_response.error.code, 400);
        assert_eq!(error_response.error.reason, "Bad Request");
        assert_eq!(error_response.error.description, "Invalid API key");

        // Testing with wrong api key
        let response = client
            .put(uri)
            .header(Header::new(
                "x-api-key",
                "8OHSw7Sllod4aVpLPC0eDw8eLTxLWml4h5altMPS4fA=",
            ))
            .body(body)
            .dispatch();
        assert_eq!(response.status(), Status::Unauthorized);
        let error_response = response.into_json::<ErrorResponse>().unwrap();
        assert_eq!(error_response.error.code, 401);
        assert_eq!(error_response.error.reason, "Unauthorized");
        assert_eq!(
            error_response.error.description,
            "The request requires user authentication."
        );
    }
}
