//! Smart Device Mobile API v1

use rocket_okapi::openapi_get_routes;

pub mod commands;
pub mod device;

/// Routes for the API v1
///
/// Routes are run through [openapi_get_routes!] to generate OpenAPI specifications from
/// implementations.
pub fn routes() -> Vec<rocket::Route> {
    openapi_get_routes![
        device::status,
        device::get_config,
        device::set_config,
        commands::factory_reset,
        commands::restart,
        commands::shutdown,
    ]
}
