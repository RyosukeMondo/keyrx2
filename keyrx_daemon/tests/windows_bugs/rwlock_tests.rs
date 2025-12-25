use keyrx_daemon::platform::windows::device_map::DeviceMap;
use std::panic;
use std::thread;

#[test]
fn test_rwlock_poison_recovery() {
    let map = DeviceMap::new();

    // 1. Thread 1 acquires write lock and panics
    let map_clone = map.clone();
    let handle = thread::spawn(move || {
        let _ = panic::catch_unwind(move || {
            let _guard = map_clone.devices.write().unwrap();
            panic!("Forceful panic while holding write lock");
        });
    });
    let _ = handle.join();

    // 2. Thread 2 attempts to acquire read lock.
    // With the fix, this should NOT panic because we use catch_unwind and proper lock handling.
    let map_clone2 = map.clone();
    let result = panic::catch_unwind(move || {
        // get() now handles lock poisoning gracefully and returns None
        map_clone2.get(0 as _);
    });

    assert!(
        result.is_ok(),
        "RwLock should handle poisoning gracefully and NOT panic"
    );
}
