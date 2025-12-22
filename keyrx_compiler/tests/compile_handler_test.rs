//! Integration test for the compile subcommand handler.

use std::fs;
use tempfile::TempDir;

use keyrx_compiler::cli::compile::handle_compile;

#[test]
fn test_handle_compile_success() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rhai");
    let output_path = temp_dir.path().join("test.krx");

    // Write a simple test script
    fs::write(
        &input_path,
        r#"
device_start("Test Keyboard");
map("VK_A", "VK_B");
device_end();
"#,
    )
    .unwrap();

    // Call handle_compile
    let result = handle_compile(&input_path, &output_path);

    // Verify success
    assert!(result.is_ok(), "Compilation should succeed");

    // Verify output file exists
    assert!(output_path.exists(), "Output file should exist");

    // Verify output file is not empty
    let bytes = fs::read(&output_path).unwrap();
    assert!(bytes.len() > 48, "Output file should contain header + data");

    // Verify magic bytes
    assert_eq!(&bytes[0..4], b"KRX\n", "Magic bytes should be correct");
}

#[test]
fn test_handle_compile_parse_error() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("invalid.rhai");
    let output_path = temp_dir.path().join("invalid.krx");

    // Write an invalid script (missing device_start)
    fs::write(
        &input_path,
        r#"
map("VK_A", "VK_B");
"#,
    )
    .unwrap();

    // Call handle_compile
    let result = handle_compile(&input_path, &output_path);

    // Verify error
    assert!(result.is_err(), "Compilation should fail");

    // Verify output file was not created
    assert!(!output_path.exists(), "Output file should not exist");
}

#[test]
fn test_handle_compile_file_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("nonexistent.rhai");
    let output_path = temp_dir.path().join("test.krx");

    // Call handle_compile without creating input file
    let result = handle_compile(&input_path, &output_path);

    // Verify error
    assert!(result.is_err(), "Compilation should fail");
}

#[test]
fn test_handle_compile_complex_config() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("complex.rhai");
    let output_path = temp_dir.path().join("complex.krx");

    // Write a more complex configuration
    fs::write(
        &input_path,
        r#"
device_start("Test Keyboard");

// Simple mapping
map("VK_A", "VK_B");

// Modifier mapping
map("VK_CapsLock", "MD_00");

// Tap-hold mapping
tap_hold("VK_Space", "VK_Space", "MD_01", 200);

// Modified output
map("VK_2", with_shift("VK_1"));

// Conditional mapping
when_start("MD_00");
map("VK_H", "VK_Left");
map("VK_J", "VK_Down");
map("VK_K", "VK_Up");
map("VK_L", "VK_Right");
when_end();

device_end();
"#,
    )
    .unwrap();

    // Call handle_compile
    let result = handle_compile(&input_path, &output_path);

    // Verify success
    assert!(
        result.is_ok(),
        "Complex compilation should succeed: {:?}",
        result.err()
    );

    // Verify output file exists
    assert!(output_path.exists(), "Output file should exist");

    // Verify output file is not empty
    let bytes = fs::read(&output_path).unwrap();
    assert!(bytes.len() > 48, "Output file should contain header + data");
}
