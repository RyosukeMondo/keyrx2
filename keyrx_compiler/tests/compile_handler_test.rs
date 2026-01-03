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
fn test_error_messages_are_user_friendly() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("syntax_error.rhai");
    let output_path = temp_dir.path().join("syntax_error.krx");

    // Write a script with a syntax error
    fs::write(
        &input_path,
        r#"
device_start("Test");
layer("base");
device_end();
"#,
    )
    .unwrap();

    // Call handle_compile
    let result = handle_compile(&input_path, &output_path);

    // Verify error occurred
    assert!(
        result.is_err(),
        "Compilation should fail due to syntax error"
    );

    // Get error message
    let error = result.unwrap_err();
    let error_message = error.to_string();

    // Verify error message is user-friendly (NOT Debug format)
    // Should NOT contain Rust debug patterns like "SyntaxError {", "file:", etc.
    assert!(
        !error_message.contains("SyntaxError {"),
        "Error message should not contain Rust debug format 'SyntaxError {{'\nGot: {}",
        error_message
    );
    assert!(
        !error_message.contains("file:"),
        "Error message should not contain debug field 'file:'\nGot: {}",
        error_message
    );
    assert!(
        !error_message.contains("line:"),
        "Error message should not contain debug field 'line:'\nGot: {}",
        error_message
    );
    assert!(
        !error_message.contains("column:"),
        "Error message should not contain debug field 'column:'\nGot: {}",
        error_message
    );
    assert!(
        !error_message.contains("import_chain:"),
        "Error message should not contain debug field 'import_chain:'\nGot: {}",
        error_message
    );

    // Should contain user-friendly information
    assert!(
        error_message.contains("syntax_error.rhai") || error_message.contains("line"),
        "Error message should contain file name or line reference\nGot: {}",
        error_message
    );
}

#[test]
fn test_parse_error_display_not_debug_format() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("unknown_function.rhai");
    let output_path = temp_dir.path().join("unknown_function.krx");

    // Write a script that calls an unknown function
    fs::write(
        &input_path,
        r#"
device_start("Test");
unknown_function("arg");
device_end();
"#,
    )
    .unwrap();

    let result = handle_compile(&input_path, &output_path);

    assert!(result.is_err(), "Should fail on unknown function");

    let error_message = result.unwrap_err().to_string();

    // Regression test: Ensure we never show "[object Object]" equivalent for Rust
    // which would be the raw Debug format like "ParseError { ... }"
    assert!(
        !error_message.contains("ParseError {"),
        "Should not expose raw Rust Debug format\nGot: {}",
        error_message
    );
    assert!(
        !error_message.contains(" { ") || !error_message.contains(": "),
        "Should not have struct-like debug formatting\nGot: {}",
        error_message
    );
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
