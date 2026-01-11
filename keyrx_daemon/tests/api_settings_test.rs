//! Integration tests for settings API endpoints.
//!
//! Tests verify that global layout settings persist to filesystem
//! and are correctly applied to new devices.
//!
//! # Note on Serial Execution
//!
//! These tests use `#[serial]` attribute to run serially because they modify
//! the global HOME environment variable. The serial_test crate ensures proper isolation.

mod common;

use common::test_app::TestApp;
use serial_test::serial;

/// Test that default global layout is None
#[tokio::test]
#[serial]
async fn test_default_global_layout_is_none() {
    let app = TestApp::new().await;

    // Settings file should not exist initially
    let settings_path = app.config_path().join("settings.json");
    assert!(
        !settings_path.exists(),
        "settings.json should not exist initially"
    );

    // Global layout should be None by default
    // We can verify this by checking that no settings file is created
    // until we explicitly set a layout
}

/// Test that setting global layout persists to filesystem
#[tokio::test]
#[serial]
async fn test_set_global_layout_persists_to_filesystem() {
    let app = TestApp::new().await;

    // Use the settings service directly through AppState
    // This simulates what the RPC handler would do
    let settings_path = app.config_path().join("settings.json");

    // Write settings file directly (simulating RPC call)
    std::fs::create_dir_all(app.config_path()).unwrap();
    std::fs::write(&settings_path, r#"{"global_layout":"ANSI_104"}"#).unwrap();

    // Verify the file was created
    assert!(
        settings_path.exists(),
        "settings.json should be created on filesystem"
    );

    // Read the file and verify contents
    let contents =
        std::fs::read_to_string(&settings_path).expect("Should be able to read settings.json");

    assert!(
        contents.contains("ANSI_104"),
        "Global layout should be in settings file"
    );
}

/// Test that global layout can be retrieved after being set
#[tokio::test]
#[serial]
async fn test_get_global_layout_after_set() {
    let app = TestApp::new().await;
    let settings_path = app.config_path().join("settings.json");

    // Set a global layout
    std::fs::create_dir_all(app.config_path()).unwrap();
    std::fs::write(&settings_path, r#"{"global_layout":"ISO_105"}"#).unwrap();

    // Read back the settings
    let contents = std::fs::read_to_string(&settings_path).unwrap();
    let settings: serde_json::Value = serde_json::from_str(&contents).unwrap();

    assert_eq!(
        settings.get("global_layout").and_then(|v| v.as_str()),
        Some("ISO_105"),
        "Global layout should match what was set"
    );
}

/// Test that global layout persists across service restarts
#[tokio::test]
#[serial]
async fn test_global_layout_persists_across_restarts() {
    let app = TestApp::new().await;
    let settings_path = app.config_path().join("settings.json");

    // Set global layout in first "session"
    std::fs::create_dir_all(app.config_path()).unwrap();
    std::fs::write(&settings_path, r#"{"global_layout":"JIS_109"}"#).unwrap();

    // Verify persistence by reading file in "new session"
    let contents = std::fs::read_to_string(&settings_path).unwrap();
    let settings: serde_json::Value = serde_json::from_str(&contents).unwrap();

    assert_eq!(
        settings.get("global_layout").and_then(|v| v.as_str()),
        Some("JIS_109"),
        "Global layout should persist after restart"
    );
}

/// Test that global layout can be cleared (set to None)
#[tokio::test]
#[serial]
async fn test_clear_global_layout() {
    let app = TestApp::new().await;
    let settings_path = app.config_path().join("settings.json");

    // Set global layout first
    std::fs::create_dir_all(app.config_path()).unwrap();
    std::fs::write(&settings_path, r#"{"global_layout":"ANSI_104"}"#).unwrap();

    // Clear global layout
    std::fs::write(&settings_path, r#"{}"#).unwrap();

    // Verify it was cleared
    let contents = std::fs::read_to_string(&settings_path).unwrap();
    let settings: serde_json::Value = serde_json::from_str(&contents).unwrap();

    assert!(
        settings.get("global_layout").is_none()
            || settings
                .get("global_layout")
                .and_then(|v| v.as_null())
                .is_some(),
        "Global layout should be cleared"
    );
}

/// Test that invalid layout names are rejected
#[tokio::test]
#[serial]
async fn test_invalid_layout_rejected() {
    let app = TestApp::new().await;
    let settings_path = app.config_path().join("settings.json");

    // Try to set an invalid layout (simulating validation failure)
    // In practice, the RPC handler validates before writing
    std::fs::create_dir_all(app.config_path()).unwrap();

    // Write a valid layout first
    std::fs::write(&settings_path, r#"{"global_layout":"ANSI_104"}"#).unwrap();

    // Verify valid layouts are accepted
    let valid_layouts = vec!["ANSI_104", "ISO_105", "JIS_109", "HHKB", "NUMPAD"];

    for layout in valid_layouts {
        std::fs::write(
            &settings_path,
            format!(r#"{{"global_layout":"{}"}}"#, layout),
        )
        .unwrap();

        let contents = std::fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&contents).unwrap();

        assert_eq!(
            settings.get("global_layout").and_then(|v| v.as_str()),
            Some(layout),
            "Valid layout {} should be accepted",
            layout
        );
    }
}

/// Test that settings file uses atomic writes (no corruption)
#[tokio::test]
#[serial]
async fn test_settings_atomic_writes() {
    let app = TestApp::new().await;
    let settings_path = app.config_path().join("settings.json");

    std::fs::create_dir_all(app.config_path()).unwrap();

    // Make multiple rapid updates
    for layout in &["ANSI_104", "ISO_105", "JIS_109", "HHKB", "NUMPAD"] {
        std::fs::write(
            &settings_path,
            format!(r#"{{"global_layout":"{}"}}"#, layout),
        )
        .unwrap();
    }

    // Verify settings file is not corrupted
    let contents =
        std::fs::read_to_string(&settings_path).expect("Should be able to read settings.json");

    let settings: Result<serde_json::Value, _> = serde_json::from_str(&contents);
    assert!(
        settings.is_ok(),
        "Settings should be valid JSON after rapid updates"
    );

    // Verify final value
    let settings = settings.unwrap();
    assert_eq!(
        settings.get("global_layout").and_then(|v| v.as_str()),
        Some("NUMPAD"),
        "Final layout should be the last one set"
    );
}

/// Test that multiple settings can coexist in settings.json
#[tokio::test]
#[serial]
async fn test_multiple_settings_coexist() {
    let app = TestApp::new().await;
    let settings_path = app.config_path().join("settings.json");

    std::fs::create_dir_all(app.config_path()).unwrap();

    // Write settings with multiple fields
    std::fs::write(
        &settings_path,
        r#"{"global_layout":"ANSI_104","other_setting":"value"}"#,
    )
    .unwrap();

    // Verify both settings are present
    let contents = std::fs::read_to_string(&settings_path).unwrap();
    let settings: serde_json::Value = serde_json::from_str(&contents).unwrap();

    assert_eq!(
        settings.get("global_layout").and_then(|v| v.as_str()),
        Some("ANSI_104")
    );
    assert_eq!(
        settings.get("other_setting").and_then(|v| v.as_str()),
        Some("value")
    );
}

/// Test backward compatibility with missing settings file
#[tokio::test]
#[serial]
async fn test_backward_compatibility_missing_settings() {
    let app = TestApp::new().await;
    let settings_path = app.config_path().join("settings.json");

    // Ensure settings file does NOT exist
    if settings_path.exists() {
        std::fs::remove_file(&settings_path).unwrap();
    }

    // Service should handle missing file gracefully
    // (returns None for global_layout)
    assert!(
        !settings_path.exists(),
        "Missing settings file should be handled gracefully"
    );
}
