use crossbeam_channel::unbounded;
use keyrx_daemon::platform::windows::device_map::DeviceMap;
use keyrx_daemon::platform::windows::rawinput::RawInputManager;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[test]
fn test_rawinput_manager_drop_safety() {
    let device_map = DeviceMap::new();
    let (tx, _rx) = unbounded();

    // We want to trigger high-frequency messages or background processing
    // and then drop the manager.
    for _ in 0..10 {
        let bridge_context = Arc::new(Mutex::new(None));
        let bridge_hook = Arc::new(Mutex::new(None));
        let manager =
            RawInputManager::new(device_map.clone(), tx.clone(), bridge_context, bridge_hook)
                .unwrap();

        // Simulate some activity/messages
        manager.simulate_raw_input(1, 0x1E, 0); // 'A' press

        // Spawn a thread that might keep some references or triggers callbacks
        let _manager_hwnd = manager.hwnd;
        thread::spawn(move || {
            // In a real scenario, wnd_proc might be called by the OS
            // For now we simulate the risk by knowing that the pointer in GWLP_USERDATA
            // is freed in Drop but wnd_proc might still be reachable or pending.
            // This test is mostly a conceptual demonstrator of the race since we can't
            // easily force the OS to call wnd_proc at the EXACT nanosecond of Drop without
            // more complex hook integration.
            thread::sleep(Duration::from_millis(1));
            // If the manager is dropped, this HWND might still be valid briefly or
            // messages might be in queue.
        });

        drop(manager);
    }
}
