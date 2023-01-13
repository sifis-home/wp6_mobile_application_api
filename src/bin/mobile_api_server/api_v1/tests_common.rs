use crate::api_common::ErrorResponse;
use crate::build_rocket;
use crate::state::DeviceState;
use mobile_api::configs::{DeviceConfig, DeviceInfo};
use mobile_api::security::SecurityKey;
use mobile_api::SifisHome;
use rocket::http::{Header, Status};
use rocket::local::blocking::Client;
use rocket::serde::uuid::Uuid;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use zbus::{dbus_interface, Connection};

pub const TEST_AUTH_KEY: SecurityKey = SecurityKey::from_bytes([
    0x52, 0x7b, 0x1e, 0x72, 0xea, 0xde, 0x4d, 0xeb, 0x2d, 0x29, 0xec, 0x94, 0xb1, 0xe3, 0xa7, 0x97,
    0x24, 0xe8, 0x4d, 0xeb, 0x2d, 0x49, 0xea, 0xef, 0x7a, 0xb1, 0x27, 0x76, 0x9a, 0x22, 0x9e, 0xdb,
]);

pub const TEST_API_KEY: &str = "UnsecureTestKeyUseOnlyToTestServerEndpoints=";

pub const TEST_DEVICE_NAME: &str = "Test Device";

pub const TEST_PRODUCT_NAME: &str = "Test Product";

pub const TEST_SHARED_DHT_KEY: SecurityKey = SecurityKey::from_bytes([
    0x4e, 0x18, 0xac, 0x22, 0xc5, 0x27, 0xb1, 0xe7, 0x2e, 0xad, 0xe0, 0xe1, 0xb4, 0xa7, 0xb2, 0x16,
    0x8a, 0xd3, 0x7a, 0xcb, 0x62, 0x9e, 0x00, 0xde, 0xbe, 0x27, 0x1e, 0x0a, 0x89, 0xdf, 0x8a, 0x0b,
]);

pub const TEST_UUID: Uuid = Uuid::from_bytes([
    0x12, 0x3e, 0x45, 0x67, 0xe8, 0x9b, 0x12, 0xd3, 0xa4, 0x56, 0x42, 0x66, 0x14, 0x17, 0x40, 0x00,
]);

pub fn api_key_header() -> Header<'static> {
    Header::new("x-api-key", TEST_API_KEY)
}

pub fn create_test_config() -> DeviceConfig {
    DeviceConfig::new(TEST_SHARED_DHT_KEY, TEST_DEVICE_NAME.to_string())
}

#[must_use]
pub fn create_test_state() -> (TempDir, DeviceState) {
    // Making SifisHome object pointing to temporary directory
    let test_dir = TempDir::new().unwrap();
    let mut sifis_home_path = PathBuf::from(test_dir.path());
    sifis_home_path.push("sifis-home");
    std::fs::create_dir_all(&sifis_home_path).unwrap();
    let sifis_home = SifisHome::new_with_path(sifis_home_path);

    // Making DeviceInfo using the SifisHome we created and saving it
    let mut private_key_path = PathBuf::from(sifis_home.home_path());
    private_key_path.push("private.pem");
    let device_info = DeviceInfo::new(
        TEST_PRODUCT_NAME.to_string(),
        TEST_AUTH_KEY,
        private_key_path,
        TEST_UUID,
    );
    sifis_home.save_info(&device_info).unwrap();

    // Making DeviceState using the above
    let device_state = DeviceState::new(sifis_home).unwrap();
    (test_dir, device_state)
}

#[must_use]
pub fn create_test_setup() -> (TempDir, Client) {
    let (test_dir, device_state) = create_test_state();
    let client = Client::tracked(build_rocket(device_state)).unwrap();
    (test_dir, client)
}

struct DbusTestingListener {
    done_tx: Option<oneshot::Sender<String>>,
}

#[dbus_interface(name = "eu.sifis_home.Testing")]
impl DbusTestingListener {
    async fn script_was_run(&mut self, script: &str) {
        if let Some(done_tx) = self.done_tx.take() {
            let _ = done_tx.send(script.to_string());
        }
    }
}

fn map_error_to_string<E>(e: E) -> String
where
    E: std::fmt::Display,
{
    e.to_string()
}

async fn wait_dbus_confirm(
    name: String,
    timeout: Duration,
    ready_tx: oneshot::Sender<()>,
) -> Result<String, String> {
    let (done_tx, done_rx) = oneshot::channel();
    let testing = DbusTestingListener {
        done_tx: Some(done_tx),
    };

    let well_known_name = format!("eu.sifis_home.Testing.{name}");

    let connection = Connection::session().await.map_err(map_error_to_string)?;
    connection
        .object_server()
        .at("/Testing", testing)
        .await
        .map_err(map_error_to_string)?;
    connection
        .request_name(well_known_name)
        .await
        .map_err(map_error_to_string)?;

    let _ = ready_tx.send(());

    match tokio::time::timeout(timeout, done_rx).await {
        Ok(ok) => ok.map_err(map_error_to_string),
        Err(err) => Err(map_error_to_string(err)),
    }
}

pub fn make_script_run_checker(
    name: &str,
    timeout: Duration,
) -> (Runtime, JoinHandle<Result<String, String>>) {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    let (ready_tx, ready_rx) = oneshot::channel();
    let handle = runtime.spawn(wait_dbus_confirm(name.to_string(), timeout, ready_tx));
    let _ = ready_rx.blocking_recv();
    (runtime, handle)
}

pub fn test_invalid_auth_get(client: &Client, uri: &str) {
    // Testing request without api key
    let response = client.get(uri).dispatch();
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
        .get(uri)
        .header(Header::new("x-api-key", "invalid key"))
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    let error_response = response.into_json::<ErrorResponse>().unwrap();
    assert_eq!(error_response.error.code, 400);
    assert_eq!(error_response.error.reason, "Bad Request");
    assert_eq!(error_response.error.description, "Invalid API key");

    // Testing with wrong api key
    let response = client
        .get(uri)
        .header(Header::new(
            "x-api-key",
            "8OHSw7Sllod4aVpLPC0eDw8eLTxLWml4h5altMPS4fA=",
        ))
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
