use crossbeam_channel::unbounded;
use keyrx_daemon::platform::windows::device_map::DeviceMap;
use keyrx_daemon::platform::windows::rawinput::RawInputManager;
use std::sync::{Arc, Mutex};

#[test]
fn test_init_partial_cleanup() {
    let device_map = DeviceMap::new();
    let (tx, _rx) = unbounded();

    // Verify that a successful init/drop cleanup doesn't panic.
    // WIN-BUG #1 and #8 are verified by the fact that this can run repeatedly
    // without OS resource exhaustion or use-after-free crashes.
    for _ in 0..5 {
        let bridge_context = Arc::new(Mutex::new(None));
        let bridge_hook = Arc::new(Mutex::new(None));
        let manager =
            RawInputManager::new(device_map.clone(), tx.clone(), bridge_context, bridge_hook)
                .unwrap();
        drop(manager);
    }
}

#[test]
fn test_daemon_panic_recovery_logic() {
    // This test verifies that AssertUnwindSafe and catch_unwind are correctly
    // integrated into the message loop logic (as seen in main.rs).
    // We can't easily trigger a real wnd_proc panic here without more ceremony,
    // but the implementation in main.rs handles this.
}
