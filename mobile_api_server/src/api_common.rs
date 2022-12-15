//! Common implementations for API endpoints

use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::Responder;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::{MediaType, RefOr, Responses};
use rocket_okapi::response::OpenApiResponderInner;
use rocket_okapi::util::{add_media_type, ensure_status_code_exists};
use schemars::schema::SchemaObject;
use schemars::JsonSchema;
use serde::Serialize;

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
/// Some endpoints have their collection of server responses, but this set is used in many.
#[derive(Responder)]
pub enum OkErrorBusyResponse {
    /// 200 OK
    #[response(status = 200, content_type = "json")]
    Ok(Json<OkResponse>),

    /// 500 Internal Server Server
    #[response(status = 500, content_type = "json")]
    Error(Json<ErrorResponse>),

    /// 503 Service Unavailable
    #[response(status = 503, content_type = "json")]
    Busy(Json<ErrorResponse>),
}

impl OpenApiResponderInner for OkErrorBusyResponse {
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        make_json_responses(vec![
            (200, gen.json_schema::<OkResponse>(), None),
            (400, gen.json_schema::<ErrorResponse>(), None),
            (422, gen.json_schema::<ErrorResponse>(), None),
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
