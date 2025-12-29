//! WASM module entry point for browser-based keyboard simulation.
//!
//! This module provides a WebAssembly interface to the keyrx_core library,
//! enabling browser-based configuration testing and event simulation.
//!
//! # Features
//! - Load Rhai configurations from source text
//! - Load pre-compiled .krx binary configurations
//! - Simulate keyboard event sequences
//! - Query simulation state
//!
//! # Architecture
//! Configurations are stored in a global CONFIG_STORE and referenced by opaque
//! ConfigHandle values. This prevents JavaScript from directly accessing Rust
//! memory and ensures thread safety.

#![cfg(feature = "wasm")]

extern crate std;

use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::vec::Vec;
use wasm_bindgen::prelude::*;

use crate::config::ConfigRoot;

// ============================================================================
// Global Configuration Storage
// ============================================================================

/// Global storage for loaded configurations.
///
/// Configurations are stored in a Vec and referenced by their index (ConfigHandle).
/// The Mutex ensures thread-safe access, though WASM is currently single-threaded.
static CONFIG_STORE: Lazy<Mutex<Vec<ConfigRoot>>> = Lazy::new(|| Mutex::new(Vec::new()));

// ============================================================================
// Public Types
// ============================================================================

/// Opaque handle to a loaded configuration.
///
/// This handle prevents JavaScript from directly accessing Rust memory.
/// The handle is an index into the CONFIG_STORE Vec.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigHandle(usize);

// ============================================================================
// Module Initialization
// ============================================================================

/// Initialize the WASM module.
///
/// This sets up the panic hook to provide better error messages in the
/// browser console. Call this before using any other WASM functions.
///
/// # Example (JavaScript)
/// ```javascript
/// import init, { wasm_init } from './pkg/keyrx_core.js';
///
/// await init();
/// wasm_init();
/// ```
#[wasm_bindgen]
pub fn wasm_init() {
    console_error_panic_hook::set_once();
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Store a configuration in the global CONFIG_STORE and return a handle.
fn store_config(config: ConfigRoot) -> ConfigHandle {
    let mut store = CONFIG_STORE.lock().unwrap();
    let index = store.len();
    store.push(config);
    ConfigHandle(index)
}

/// Retrieve a configuration from the CONFIG_STORE by handle.
///
/// Returns an error if the handle is invalid.
fn get_config(handle: ConfigHandle) -> Result<ConfigRoot, JsValue> {
    let store = CONFIG_STORE.lock().unwrap();
    store
        .get(handle.0)
        .cloned()
        .ok_or_else(|| JsValue::from_str(&format!("Invalid ConfigHandle: {}", handle.0)))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_handle_copy() {
        let handle1 = ConfigHandle(5);
        let handle2 = handle1;
        assert_eq!(handle1, handle2);
    }

    #[test]
    fn test_invalid_handle() {
        let invalid_handle = ConfigHandle(9999);
        let result = get_config(invalid_handle);
        assert!(result.is_err());
    }
}
