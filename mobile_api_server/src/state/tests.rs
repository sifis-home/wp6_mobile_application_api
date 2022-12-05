use crate::api_v1::tests_common::make_test_device_state;
use crate::state::BusyGuard;

// Test ignored for Miri because the server has time and io-related
// functions that are not available in isolation mode
#[cfg_attr(miri, ignore)]
#[test]
fn test_busy_guard() {
    // Shouldn't be busy at start
    let state = make_test_device_state();
    assert_eq!(state.busy(), "");

    // Making "server" busy
    let busy_message = "Testing BusyGuard";
    {
        let guard = BusyGuard::try_busy(&state, busy_message);
        assert!(guard.is_ok());
        assert_eq!(state.busy(), busy_message);

        // Second guard should also fail with the busy message
        let result = BusyGuard::try_busy(&state, busy_message);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), busy_message);
    }

    // Busy guard went out of scope, "server" should be free now.
    let state = make_test_device_state();
    assert_eq!(state.busy(), "");
}
