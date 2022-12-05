//! Helper application to create `device.json` for the server.
//!
//! This application creates a new device.json file. The file is written to the `/opt/sifis-home/`
//! path by default, but the location can be changed with the `SIFIS_HOME_PATH` environment
//! variable or with the -o option.

use clap::Parser;
use mobile_api::configs::DeviceInfo;
use mobile_api::device_info_path;
use qrcodegen::{QrCode, QrCodeEcc, QrSegment};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

/// Command line arguments for the application
///
/// We use clap crate with 'derive' feature to generate command line arguments from this struct.
#[derive(Debug, Parser)]
#[command(about = "Creates 'device.json' for the server")]
#[command(
    long_about = "This application creates a new device.json file. The file is written to
the `/opt/sifis-home/` path by default, but the location can be changed
with the `SIFIS_HOME_PATH` environment variable or with the -o option."
)]
struct Arguments {
    /// Product name for the SIFIS-Home Smart Device
    product_name: String,

    /// Sets a custom output path
    #[arg(short, long, value_name = "PATH")]
    output_path: Option<PathBuf>,

    /// Force writing of a new device.json file
    #[arg(short, long)]
    force: bool,

    /// Set a custom path for the private key
    #[arg(short, long, value_name = "FILE")]
    private_key: Option<PathBuf>,

    /// Write authorization key to QR code as SVG image
    #[arg(short, long, value_name = "FILE")]
    save_qr_code_svg: Option<PathBuf>,
}

fn main() -> ExitCode {
    // Parse command line arguments
    let arguments = Arguments::parse();

    // Load .env if available
    if dotenv::dotenv().is_ok() {
        println!("Loaded environment variables from .env file");
    }

    // Check if output path option is given or use default path
    let device_info_file = match arguments.output_path {
        Some(mut path) => {
            path.push("device.json");
            path
        }
        None => device_info_path(),
    };

    // Stop if the device.json file already exists and force option is not given
    if device_info_file.exists() && !arguments.force {
        println!(
            "The device information file already exists at: {:?}",
            device_info_file
        );
        println!("You can use the -f option to overwrite it with a new one.");
        return ExitCode::SUCCESS;
    }

    // Ensure that the output path exists
    let output_path = device_info_file
        .parent()
        .expect("Could not get output path");
    if let Err(err) = fs::create_dir_all(output_path) {
        eprintln!("Could not create output path: {}", err);
        return ExitCode::FAILURE;
    }

    // Create device info and update the private key path if it was given
    let mut device_info =
        DeviceInfo::new(arguments.product_name).expect("Could not create a new device info");
    if let Some(private_key) = arguments.private_key {
        device_info.set_private_key_file(private_key);
    }

    // Try to save device info
    if let Err(err) = device_info.save_to(&device_info_file) {
        eprintln!("Could not write device information: {}", err);
        return ExitCode::FAILURE;
    };
    println!(
        "A new device information file was written to: {:?}",
        device_info_file
    );

    // Create Qr Code image?
    if let Some(svg_file) = arguments.save_qr_code_svg {
        // We store authorization key as hex string to the Qr Code
        let segments = QrSegment::make_segments(&device_info.authorization_key().hex(true));
        let qr_code = match QrCode::encode_segments(&segments, QrCodeEcc::Quartile) {
            Ok(code) => code,
            Err(err) => {
                eprintln!("Could not create Qr Code: {}", err);
                return ExitCode::FAILURE;
            }
        };
        let svg = to_svg_string(&qr_code, 4);
        match fs::write(&svg_file, &svg) {
            Ok(_) => println!("Qr Code saved as: {:?}", svg_file),
            Err(err) => {
                eprintln!("Could not save Qr Code: {}", err);
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}

/// Returns a string of SVG code for an image depicting
/// the given QR Code, with the given number of border modules.
/// The string always uses Unix newlines (\n), regardless of the platform.
fn to_svg_string(qr: &QrCode, border: i32) -> String {
    assert!(border >= 0, "Border must be non-negative");
    let mut result = String::new();
    result += "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
    result += "<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\">\n";
    let dimension = qr
        .size()
        .checked_add(border.checked_mul(2).unwrap())
        .unwrap();
    result += &format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" viewBox=\"0 0 {0} {0}\" stroke=\"none\">\n", dimension);
    result += "\t<rect width=\"100%\" height=\"100%\" fill=\"#FFFFFF\"/>\n";
    result += "\t<path d=\"";
    for y in 0..qr.size() {
        for x in 0..qr.size() {
            if qr.get_module(x, y) {
                if x != 0 || y != 0 {
                    result += " ";
                }
                result += &format!("M{},{}h1v1h-1z", x + border, y + border);
            }
        }
    }
    result += "\" fill=\"#000000\"/>\n";
    result += "</svg>\n";
    result
}
