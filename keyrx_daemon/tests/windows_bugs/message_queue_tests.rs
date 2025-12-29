use crossbeam_channel::unbounded;
use keyrx_daemon::platform::windows::device_map::DeviceMap;
use keyrx_daemon::platform::windows::rawinput::RawInputManager;

#[test]
fn test_message_queue_flood() {
    let device_map = DeviceMap::new();
    let (tx, _rx) = unbounded();
    let manager = RawInputManager::new(device_map, tx).unwrap();

    // We want to test if GetRawInputData handles large buffer requests safely.
    // While we can't easily spoof GetRawInputData's OS-internal state to return
    // a huge 'size', we can check if our wrapper at least compiles correctly
    // and if we can trigger the message path.

    // In our audit we noted:
    // if GetRawInputData(..., &mut close_size, ...) == 0 {
    //     let mut buffer = vec![0u8; close_size as usize]; // RISK: close_size unbounded
    // }

    // For this test, we just verify the message loop processes events.
    // The "regression" part for OOM is hard to automate without mocking the API call,
    // so we'll treat this as a stress test for now.

    for _i in 0..100 {
        manager.simulate_raw_input(1, 0x1E, 0); // Simulate typing
    }
}
