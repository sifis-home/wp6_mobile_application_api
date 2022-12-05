use crate::api_v1::tests_common::make_test_client;
use rocket::http::Status;

// Test ignored for Miri because the server has time and io-related
// functions that are not available in isolation mode
#[cfg_attr(miri, ignore)]
#[test]
fn test_factory_reset() {
    let client = make_test_client();
    let response = client.get("/v1/command/factory_reset").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.into_string().unwrap(),
        concat!(
            "To perform a factory reset, the `confirm` parameter must be set to the ",
            "message `I really want to perform a factory reset`"
        )
    );
}

// Test ignored for Miri because the server has time and io-related
// functions that are not available in isolation mode
#[cfg_attr(miri, ignore)]
#[test]
fn test_restart() {
    let client = make_test_client();
    let response = client.get("/v1/command/restart").dispatch();
    assert_eq!(response.status(), Status::Ok);
}

// Test ignored for Miri because the server has time and io-related
// functions that are not available in isolation mode
#[cfg_attr(miri, ignore)]
#[test]
fn test_shutdown() {
    let client = make_test_client();
    let response = client.get("/v1/command/shutdown").dispatch();
    assert_eq!(response.status(), Status::Ok);
}
