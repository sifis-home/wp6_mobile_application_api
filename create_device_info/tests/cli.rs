use assert_cmd::prelude::*;
use image::DynamicImage;
use mobile_api::configs::DeviceInfo;
use mobile_api::security::SecurityKey;
use predicates::prelude::*;
use resvg::{tiny_skia, usvg};
use std::path::Path;
use std::{error::Error, fs, os::unix::fs::PermissionsExt, path::PathBuf, process::Command};
use tempfile::TempDir;

const APP_NAME: &str = "create_device_info";

// Test ignored for miri, because file operations are not available when isolation is enabled.
#[cfg_attr(miri, ignore)]
#[test]
fn test_errors_with_output_path() -> Result<(), Box<dyn Error>> {
    if users::get_current_uid() == 0 {
        println!("Warning: skipping this test because the permission check doesn't work for root.");
        return Ok(());
    }

    // Making directory without permissions
    let tmp_dir = TempDir::new()?;
    fs::set_permissions(tmp_dir.path(), fs::Permissions::from_mode(0o000))?;

    // App should give error when trying to create a new directory to write protected one
    let mut new_dir = PathBuf::from(tmp_dir.path());
    new_dir.push("new");
    let mut command = Command::cargo_bin(APP_NAME)?;
    command
        .arg("--output-path")
        .arg(&new_dir)
        .arg("\"Test device\"");
    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("Could not create output path"));

    // Running the app should give error when trying to write directory without permission
    let mut command = Command::cargo_bin(APP_NAME)?;
    command
        .arg("--output-path")
        .arg(tmp_dir.path())
        .arg("\"Test device\"");
    command.assert().failure().stderr(predicate::str::contains(
        "Could not write device information",
    ));

    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)] // File operations not available for miri when isolation is enabled
fn test_should_not_overwrite_by_default() -> Result<(), Box<dyn Error>> {
    // First write to tmp dir should work
    let tmp_dir = TempDir::new()?;
    let mut command = Command::cargo_bin(APP_NAME)?;
    command
        .arg("--output-path")
        .arg(tmp_dir.path())
        .arg("\"Test device\"");
    command.assert().success().stdout(predicate::str::contains(
        "A new device information file was written to:",
    ));

    // Second run should stop and tell about already existing file and about force option
    command
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "The device information file already exists at:",
        ))
        .stdout(predicate::str::contains(
            "You can use the -f option to overwrite it with a new one.",
        ));

    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)] // File operations not available for miri when isolation is enabled
fn test_forcing_overwrite() -> Result<(), Box<dyn Error>> {
    // First write to tmp dir should work
    let tmp_dir = TempDir::new()?;
    let mut command = Command::cargo_bin(APP_NAME)?;
    command
        .arg("--output-path")
        .arg(tmp_dir.path())
        .arg("\"Test device\"");
    command.assert().success().stdout(predicate::str::contains(
        "A new device information file was written to:",
    ));

    // Second write should also work when --force option is given
    let mut command = Command::cargo_bin(APP_NAME)?;
    command
        .arg("--force")
        .arg("--output-path")
        .arg(tmp_dir.path())
        .arg("\"Test device\"");
    command.assert().success().stdout(predicate::str::contains(
        "A new device information file was written to:",
    ));

    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)] // File operations not available for miri when isolation is enabled
fn test_private_key() -> Result<(), Box<dyn Error>> {
    // Writing a new device info with custom private key path should be success
    let tmp_dir = TempDir::new()?;
    let mut private_key_file = PathBuf::from(tmp_dir.path());
    private_key_file.push("private.key");
    let mut command = Command::cargo_bin(APP_NAME)?;
    command
        .arg("--private-key")
        .arg(&private_key_file)
        .arg("--output-path")
        .arg(tmp_dir.path())
        .arg("\"Test device\"");
    command.assert().success();

    // We should be able to load device info and it should have our custom private key
    let mut device_info_file = PathBuf::from(tmp_dir.path());
    device_info_file.push("device.json");
    let device_info = DeviceInfo::load_from(&device_info_file).unwrap();
    assert_eq!(device_info.private_key_file(), &private_key_file);

    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)] // File operations not available for miri when isolation is enabled
fn test_authorization_key_in_qrcode() -> Result<(), Box<dyn Error>> {
    // SVG generation should work
    let tmp_dir = TempDir::new()?;
    let mut svg_file = PathBuf::from(tmp_dir.path());
    svg_file.push("code.svg");
    let mut command = Command::cargo_bin(APP_NAME)?;
    command
        .arg("--save-qr-code-svg")
        .arg(&svg_file)
        .arg("--output-path")
        .arg(tmp_dir.path())
        .arg("\"Test device\"");
    command
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "A new device information file was written to:",
        ))
        .stdout(predicate::str::contains("Qr Code saved as:"));

    // SVG file should exists in tmp dir
    assert!(svg_file.exists());

    // Render SVG to image and decode it with Qr decoder
    let luma_image = svg_to_dynamic_image(&svg_file)?.into_luma8();
    let mut prepared_image = rqrr::PreparedImage::prepare(luma_image);
    let grids = prepared_image.detect_grids();
    assert_eq!(grids.len(), 1);
    let (_, authorization_key_string) = grids[0].decode()?;

    // Converting hex string to SecurityKey
    let authorization_key = SecurityKey::from_hex(authorization_key_string.as_str()).unwrap();

    // Reading the device info so that we can check that the generated SVG contains correct
    // authorization key
    let mut device_info_file = PathBuf::from(tmp_dir.path());
    device_info_file.push("device.json");
    let device_info = DeviceInfo::load_from(&device_info_file).unwrap();

    // The key from the Qr code should be identical to one from the device info file
    assert_eq!(&authorization_key, device_info.authorization_key());

    Ok(())
}

fn svg_to_dynamic_image(file: &Path) -> Result<DynamicImage, Box<dyn Error>> {
    // Rendering SVG to pixmap
    let mut svg_options = usvg::Options {
        resources_dir: Some(PathBuf::from(file.parent().unwrap())),
        ..Default::default()
    };
    svg_options.fontdb.load_system_fonts();
    let svg_data = fs::read(file).unwrap();
    let svg_tree = usvg::Tree::from_data(&svg_data, &svg_options.to_ref())?;
    let size = svg_tree.size.width() as u32 * 4;
    let mut pixmap = tiny_skia::Pixmap::new(size, size).unwrap();
    resvg::render(
        &svg_tree,
        usvg::FitTo::Size(size, size),
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();

    // Pixmap -> RgbaImage -> DynamicImage
    let rgba_image = image::RgbaImage::from_raw(size, size, Vec::from(pixmap.data())).unwrap();
    let dynamic_image = image::DynamicImage::from(rgba_image);
    Ok(dynamic_image)
}
