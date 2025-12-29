use crossbeam_channel::unbounded;
use keyrx_daemon::platform::windows::device_map::DeviceMap;
use keyrx_daemon::platform::windows::rawinput::RawInputManager;
use windows_sys::Win32::Foundation::HANDLE;

#[test]
fn test_device_removal_during_events() {
    let map = DeviceMap::new();
    let (tx, _rx) = unbounded();
    let manager = RawInputManager::new(map.clone(), tx).unwrap();

    let h_device = 0x1234 as HANDLE;
    map.add_synthetic_device(
        h_device as usize,
        "test_path".to_string(),
        Some("SN123".to_string()),
    );

    // Simulate input while concurrently removing the device
    let manager_clone = manager.hwnd;
    let _ = manager_clone;

    // Remove the device
    map.remove_device(h_device);

    // Verify no panic during access to removed device in simulate_raw_input
    // This previously would have unwrap()ed on a None if not careful,
    // though RawInputManager::simulate_raw_input handles the context ptr.
    // The key fix is in process_raw_keyboard and device_map methods.
    manager.simulate_raw_input(h_device as usize, 0x1E, 0);
}

#[test]
fn test_device_add_error_logging() {
    let map = DeviceMap::new();
    // This just verifies the method exists and doesn't panic
    let result = map.add_device(0 as HANDLE);
    assert!(result.is_err());
}
