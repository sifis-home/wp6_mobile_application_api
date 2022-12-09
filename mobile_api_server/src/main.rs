//! Smart Device Mobile API Service
//!
//! This program contains the Rocket server, which provides an interface that the mobile application
//! can use to initialize the device as part of the SIFIS-Home network.
//!
//! The following environment variables change the behavior of this server program.
//!
//! * `SIFIS_HOME_PATH` - The path where the device settings are stored
//! * `MOBILE_API_SCRIPTS_PATH` - The path where command scripts are stored
//! * `ROCKET_ADDRESS` - Ip address or host to listen on
//! * `ROCKET_PORT` - Port number to listen on
//!
//! These environment variables can be set in the `.env` file. This file is used during the
//! development to store configurations in the program's local directory.
//!
//! See more Rocket related configuration options from: [rocket#configuration]

use crate::state::DeviceState;
use mobile_api::configs::DeviceInfo;
use mobile_api::error::ErrorKind;
use mobile_api::{device_info_path, sifis_home_path};
use rocket::fs::{relative, FileServer};
use rocket::{Build, Rocket};
use rocket_okapi::rapidoc::{make_rapidoc, GeneralConfig, HideShowConfig, RapiDocConfig};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use std::process::ExitCode;

pub mod api_common;
pub mod api_v1;
pub mod device_status;
pub mod state;

/// Entry Point for the Server Program
#[rocket::main]
async fn main() -> ExitCode {
    // Read .env file when available
    if dotenv::dotenv().is_ok() {
        println!("Loaded environment variables from .env file");
    }
    println!(
        "SIFIS-Home path: {}",
        &sifis_home_path()
            .to_str()
            .expect("Could not get SIFIS-Home path")
    );

    // Try to load device info and use it to create device state
    let device_info = match DeviceInfo::load() {
        Ok(device_info) => device_info,
        Err(error) => {
            // Special message for file not found error
            if let ErrorKind::IoError(io_error) = error.kind() {
                if io_error.kind() == std::io::ErrorKind::NotFound {
                    eprintln!(
                        "Device information file {:?} not found.",
                        device_info_path()
                    );
                    eprintln!("You can use create_device_info application to create it.");
                    return ExitCode::FAILURE;
                }
            };

            // Error message for any other error
            eprintln!(
                "Could not load device information file: {:?}",
                device_info_path()
            );
            eprintln!("{}", error);
            return ExitCode::FAILURE;
        }
    };
    let device_state = DeviceState::new(device_info);

    let launch_result = build_rocket(device_state).launch().await;

    // Check launch result
    match launch_result {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Rocket had an error: {}", err);
            ExitCode::FAILURE
        }
    }
}

/// Builds Mobile API Rocket
///
/// This function creates a Rocket object that is ready to launch. Rocket is created from the main
/// function, but also unit tests use this function to check endpoints using local instances.
fn build_rocket(state: DeviceState) -> Rocket<Build> {
    // Prepare configuration for API documentation.
    let rapidoc_config = RapiDocConfig {
        title: Some("Smart Device Mobile API | Documentation".to_string()),
        general: GeneralConfig {
            spec_urls: vec![UrlObject::new("General", "../openapi.json")],
            ..Default::default()
        },
        hide_show: HideShowConfig {
            allow_spec_url_load: false,
            allow_spec_file_load: false,
            ..Default::default()
        },
        ..Default::default()
    };
    let swagger_ui_config = SwaggerUIConfig {
        url: "../openapi.json".to_owned(),
        ..Default::default()
    };

    // Launch server
    rocket::build()
        // Manage state through DeviceState object
        .manage(state)
        // Mount static files to root
        .mount("/", FileServer::from(relative!("static")))
        // Mount APIv1
        .mount("/v1/", api_v1::routes())
        // API documentation from the implementation
        .mount("/v1/rapidoc/", make_rapidoc(&rapidoc_config))
        .mount("/v1/swagger-ui/", make_swagger_ui(&swagger_ui_config))
}
