use std::fs;
use std::path::Path;

#[test]
fn test_windows_api_error_handling_completeness() {
    let rawinput_rs =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/platform/windows/rawinput.rs");
    let content = fs::read_to_string(rawinput_rs).unwrap();

    // Check for RegisterClassExW error handling
    assert!(content.contains("if RegisterClassExW(&wnd_class) == 0"));

    // Check for CreateWindowExW error handling
    assert!(content.contains("if hwnd == 0 as _"));

    // Check for RegisterRawInputDevices error handling
    assert!(content
        .contains("if RegisterRawInputDevices(&rid, 1, size_of::<RAWINPUTDEVICE>() as u32) == 0"));
}

#[test]
fn test_send_input_error_handling() {
    let inject_rs = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/platform/windows/inject.rs");
    let content = fs::read_to_string(inject_rs).unwrap();

    // Check for SendInput return value validation
    assert!(content.contains("if SendInput(1, &input, size_of::<INPUT>() as i32) == 0"));
    assert!(
        content.contains("log::error!(\"SendInput failed: {}\", std::io::Error::last_os_error())")
    );
}

#[test]
fn test_scancode_mapping_table_completeness() {
    // Verify that we have a significant number of keys mapped
    // (This is a heuristic test)
    let keycode_rs = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/platform/windows/keycode.rs");
    let content = fs::read_to_string(keycode_rs).unwrap();

    // Check if common keys are mapped
    assert!(content.contains("KeyCode::A"));
    assert!(content.contains("KeyCode::LShift"));
    assert!(content.contains("KeyCode::Enter"));
}
