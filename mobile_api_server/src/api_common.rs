//! Common implementations for API endpoints

use crate::api_common::ApiKeyError::{InvalidKey, WrongKey};
use crate::state::DeviceState;
use mobile_api::security::SecurityKey;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{Request, Responder};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::{
    MediaType, Object, RefOr, Responses, SecurityRequirement, SecurityScheme, SecuritySchemeData,
};
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use rocket_okapi::response::OpenApiResponderInner;
use rocket_okapi::util::{add_media_type, ensure_status_code_exists};
use schemars::schema::SchemaObject;
use schemars::JsonSchema;
use serde::Serialize;

/// ApiKey is the authentication code from Qr Code
#[derive(Debug)]
pub struct ApiKey;

/// Possible values returned if ApiKey validation fails
#[derive(Debug)]
pub enum ApiKeyError {
    /// The provided key was in an invalid format or the wrong size
    InvalidKey(Json<ErrorResponse>),

    /// The provided key was in valid format but was incorrect
    WrongKey(Json<ErrorResponse>),
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ApiKeyError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("x-api-key") {
            // Response for a missing key
            None => Outcome::Failure((
                Status::BadRequest,
                InvalidKey(ErrorResponse::bad_request(Some(
                    "Missing `x-api-key` header.",
                ))),
            )),

            // We have key, checking if it valid and correct
            Some(given_key_str) => match SecurityKey::from_string(given_key_str) {
                Ok(key) => {
                    // Key is valid, but is it correct?
                    let state = request
                        .rocket()
                        .state::<DeviceState>()
                        .expect("state object should always be available");
                    if state.device_info().authorization_key() == &key {
                        // Yes, access should be granted
                        Outcome::Success(ApiKey)
                    } else {
                        // No, access should be denied
                        Outcome::Failure((
                            Status::Unauthorized,
                            WrongKey(ErrorResponse::unauthorized(None)),
                        ))
                    }
                }

                // Key was invalid
                Err(_) => Outcome::Failure((
                    Status::BadRequest,
                    InvalidKey(ErrorResponse::bad_request(Some("Invalid API key"))),
                )),
            },
        }
    }
}

impl<'a> OpenApiFromRequest<'a> for ApiKey {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        let security_scheme = SecurityScheme {
            description: Some(
                concat!("## Requires an API key to access.\n",
                "The key is in the Qr code and can be sent as a hex string or base64 format.\n\n",
                "### Hex string example:\n",
                "`x-api-key: f0e1d2c3b4a5968778695a4b3c2d1e0f0f1e2d3c4b5a69788796a5b4c3d2e1f0`\n\n",
                "### Base64 example:\n",
                "`x-api-key: 8OHSw7Sllod4aVpLPC0eDw8eLTxLWml4h5altMPS4fA=`\n\n",
                "**Note:** These are examples and therefore incorrect.\n\n",
                "---")
                .to_string(),
            ),
            data: SecuritySchemeData::ApiKey {
                name: "x-api-key".to_string(),
                location: "header".to_string(),
            },
            extensions: Object::default(),
        };
        let mut security_req = SecurityRequirement::new();
        security_req.insert("ApiKeyAuth".to_string(), Vec::new());
        Ok(RequestHeaderInput::Security(
            "ApiKeyAuth".to_owned(),
            security_scheme,
            security_req,
        ))
    }
}

/// Server error response content
#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ErrorResponseContent {
    /// Status code
    pub code: u16,

    /// Error reason
    pub reason: String,

    /// Error message
    pub description: String,
}

/// Server error response message
#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ErrorResponse {
    /// Error content
    pub error: ErrorResponseContent,
}

impl ErrorResponse {
    /// Constructing `400 Bad Request` Response
    ///
    /// The `description` option allows custom description,
    /// but a default description is used by giving a `None` value.
    pub fn bad_request(description: Option<&str>) -> Json<ErrorResponse> {
        Json(ErrorResponse {
            error: ErrorResponseContent {
                code: 400,
                reason: "Bad Request".to_string(),
                description: description
                    .unwrap_or("The request could not be understood by the server due to malformed syntax.")
                    .to_string(),
            },
        })
    }

    /// Constructing `401 Unauthorized` Response
    ///
    /// The `description` option allows custom description,
    /// but a default description is used by giving a `None` value.
    pub fn unauthorized(description: Option<&str>) -> Json<ErrorResponse> {
        Json(ErrorResponse {
            error: ErrorResponseContent {
                code: 401,
                reason: "Unauthorized".to_string(),
                description: description
                    .unwrap_or("The request requires user authentication.")
                    .to_string(),
            },
        })
    }

    /// Constructing `404 Not Found` Response
    ///
    /// The `description` option allows custom description,
    /// but a default description is used by giving a `None` value.
    pub fn not_found(description: Option<&str>) -> Json<ErrorResponse> {
        Json(ErrorResponse {
            error: ErrorResponseContent {
                code: 404,
                reason: "Not Found".to_string(),
                description: description
                    .unwrap_or("The requested resource could not be found.")
                    .to_string(),
            },
        })
    }

    /// Constructing `500 Internal Server Error` Response
    ///
    /// This response should be used only for unexpected errors.
    /// The `description` should contain a message of what went wrong.
    pub fn internal_server_error(description: String) -> Json<ErrorResponse> {
        Json(ErrorResponse {
            error: ErrorResponseContent {
                code: 500,
                reason: "Internal Server Error".to_string(),
                description,
            },
        })
    }

    /// Constructing `503 Service Unavailable` Response
    ///
    /// The `description` should contain a message of why server is busy.
    pub fn service_unavailable(description: &str) -> Json<ErrorResponse> {
        Json(ErrorResponse {
            error: ErrorResponseContent {
                code: 503,
                reason: "Service Unavailable".to_string(),
                description: description.to_string(),
            },
        })
    }
}

/// Operation complete message
#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct OkResponse {
    /// Status code
    pub code: u16,

    /// Description message
    pub message: String,
}

impl OkResponse {
    /// Constructor for `200 OK` Response
    ///
    /// Message should tell what was done.
    pub fn message(message: &'static str) -> Json<OkResponse> {
        Json(OkResponse {
            code: 200,
            message: message.to_string(),
        })
    }
}

/// A general set of server responses
///
/// Some endpoints have their collection of server responses, but these are used in many.
#[derive(Responder)]
pub enum GenericResponse {
    /// 200 OK
    #[response(status = 200, content_type = "json")]
    Ok(Json<OkResponse>),

    /// 400 Bad Request
    #[response(status = 400, content_type = "json")]
    BadRequest(Json<ErrorResponse>),

    /// 401 Unauthorized
    #[response(status = 401, content_type = "json")]
    Unauthorized(Json<ErrorResponse>),

    /// 500 Internal Server Server
    #[response(status = 500, content_type = "json")]
    Error(Json<ErrorResponse>),

    /// 503 Service Unavailable
    #[response(status = 503, content_type = "json")]
    Busy(Json<ErrorResponse>),
}

impl OpenApiResponderInner for GenericResponse {
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        make_json_responses(vec![
            (200, gen.json_schema::<OkResponse>(), None),
            (400, gen.json_schema::<ErrorResponse>(), None),
            (401, gen.json_schema::<ErrorResponse>(), None),
            (500, gen.json_schema::<ErrorResponse>(), None),
            (503, gen.json_schema::<ErrorResponse>(), None),
        ])
    }
}

/// Responses Generator
///
/// This responses generator allows an efficient way to implement [OpenApiResponderInner] for
/// responses. In addition, the function automatically adds descriptions for some known status
/// response codes.
///
/// # Example
/// ```rust
/// impl OpenApiResponderInner for FactoryResetResponse {
///     fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
///         make_json_responses(vec![
///             (200, gen.json_schema::<OkResponse>(), None),
///             (400, gen.json_schema::<ErrorResponse>(), None),
///             (500, gen.json_schema::<ErrorResponse>(), None),
///             (503, gen.json_schema::<ErrorResponse>(), None),
///         ])
///     }
/// }
/// ```
pub fn make_json_responses(
    status_schema_description: Vec<(u16, SchemaObject, Option<&str>)>,
) -> rocket_okapi::Result<Responses> {
    let mut responses = Responses::default();
    for (status, schema, description) in status_schema_description {
        let mut response = match ensure_status_code_exists(&mut responses, status) {
            RefOr::Ref(_) => continue, // Skipping references
            RefOr::Object(object) => object,
        };
        response.description = match description {
            None => match status {
                // Default descriptions for known status codes
                200 => "Ok",
                400 => "Bad Request",
                401 => "Unauthorized",
                404 => "Not Found",
                422 => "Unprocessable Entity",
                500 => "Internal Server Error",
                503 => "Service Unavailable",
                _ => "",
            },
            Some(description) => description,
        }
        .to_string();
        let media = MediaType {
            schema: Some(schema),
            ..MediaType::default()
        };
        add_media_type(&mut response.content, "application/json", media);
    }
    Ok(responses)
}
