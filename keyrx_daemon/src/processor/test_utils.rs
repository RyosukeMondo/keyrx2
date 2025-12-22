//! Test utilities for processor tests.

use keyrx_core::config::mappings::{DeviceConfig, DeviceIdentifier, KeyMapping};

/// Helper to create a simple test config with given mappings.
pub fn create_test_config(mappings: Vec<KeyMapping>) -> DeviceConfig {
    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: String::from("*"),
        },
        mappings,
    }
}
