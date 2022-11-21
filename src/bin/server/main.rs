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

use rocket::fs::{relative, FileServer};
use rocket_okapi::rapidoc::{make_rapidoc, GeneralConfig, HideShowConfig, RapiDocConfig};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};

pub mod api_v1;

/// Entry Point for the Server Program
#[rocket::main]
async fn main() {
    // Read .env file when available
    if dotenv::dotenv().is_ok() {
        println!("Loaded environment variables from .env file");
    }
    println!(
        "SIFIS-Home path: {}",
        &mobile_api::sifis_home_path()
            .to_str()
            .expect("Could not get SIFIS-Home path")
    );

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
