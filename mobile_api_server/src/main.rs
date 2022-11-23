//! Smart Device Mobile API Service
//!
//! This program contains the Rocket server, which provides an interface that the mobile application
//! can use to initialize the device as part of the SIFIS-Home network.
//!
//! The following environment variables change the behavior of this server program.
//!
//! * `SIFIS_HOME_PATH` - The path where the device settings are stored
//! * `ROCKET_ADDRESS` - Ip address or host to listen on
//! * `ROCKER_PORT` - Port number to listen on
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
use rocket_okapi::rapidoc::{make_rapidoc, GeneralConfig, HideShowConfig, RapiDocConfig};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use std::fs;

pub mod api_v1;
pub mod device_status;
pub mod state;

/// Loads device info or creates new one
///
/// If a new info file is created, the user must at least change
/// the product name inside the file before the server can start.
fn load_device_info() -> Result<DeviceInfo, String> {
    // Ensure that SIFIS-Home path exists
    fs::create_dir_all(sifis_home_path())
        .map_err(|error| format!("Could not create SIFIS-Home path: {}", error))?;

    // Try to load existing information file
    let device_info = match DeviceInfo::load() {
        Ok(info) => info,
        Err(load_error) => match load_error.kind() {
            ErrorKind::IoError(io_error) => match io_error.kind() {
                std::io::ErrorKind::NotFound => {
                    // Creating new device info with EDIT ME as product name
                    let device_info = DeviceInfo::new(String::from("EDIT ME")).map_err(|err| {
                        format!("Unexpected error when creating a new DeviceInfo: {}", err)
                    })?;
                    device_info
                        .save()
                        .map_err(|err| format!("Could not create a new device info: {}", err))?;
                    device_info
                }
                _ => return Err(format!("Could not load device info: {}", io_error)),
            },
            _ => return Err(format!("Could not load device info: {}", load_error)),
        },
    };

    // Check for EDIT ME product name
    if device_info.product_name() == "EDIT ME" {
        return Err(format!(
            "Please edit the {:?} file and change the product name.",
            device_info_path()
        ));
    }
    Ok(device_info)
}

/// Entry Point for the Server Program
#[rocket::main]
async fn main() {
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
    let device_info = match load_device_info() {
        Ok(device_info) => device_info,
        Err(message) => {
            eprintln!("{}", message);
            return;
        }
    };
    let device_state = DeviceState::new(device_info);

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
    let launch_result = rocket::build()
        // Manage state through DeviceState object
        .manage(device_state)
        // Mount static files to root
        .mount("/", FileServer::from(relative!("static")))
        // Mount APIv1
        .mount("/v1/", api_v1::routes())
        // API documentation from the implementation
        .mount("/v1/rapidoc/", make_rapidoc(&rapidoc_config))
        .mount("/v1/swagger-ui/", make_swagger_ui(&swagger_ui_config))
        .launch()
        .await;

    // Check launch result
    match launch_result {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Rocket had an error: {}", err);
        }
    };
}
