//! Rhai DSL parser for keyrx configuration.
//!
//! This module provides a complete Rhai DSL parser that can be used by both
//! the compiler (keyrx_compiler) and WASM (keyrx_core wasm feature).
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::parser::Parser;
//!
//! let source = r#"
//!     device_start("*");
//!     map("VK_A", "VK_B");
//!     device_end();
//! "#;
//!
//! let parser = Parser::new();
//! let config = parser.parse_string(source)?;
//! ```

pub mod error;
pub mod functions;
pub mod state;
pub mod validators;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use rhai::{Engine, Scope};
use sha2::{Digest, Sha256};
use spin::Mutex;

use crate::config::{ConfigRoot, Metadata, Version};
use state::ParserState;

/// Main parser for Rhai DSL.
pub struct Parser {
    engine: Engine,
    state: Arc<Mutex<ParserState>>,
}

impl Parser {
    /// Create a new parser with all functions registered.
    pub fn new() -> Self {
        let mut engine = Engine::new();
        let state = Arc::new(Mutex::new(ParserState::new()));

        // Set resource limits
        engine.set_max_operations(100_000);
        engine.set_max_expr_depths(100, 100);
        engine.set_max_call_levels(100);

        // Register all DSL functions
        functions::device::register_device_functions(&mut engine, Arc::clone(&state));
        functions::map::register_map_functions(&mut engine, Arc::clone(&state));
        functions::tap_hold::register_tap_hold_function(&mut engine, Arc::clone(&state));
        functions::conditional::register_when_functions(&mut engine, Arc::clone(&state));
        functions::modifiers::register_modifier_functions(&mut engine);

        Self { engine, state }
    }

    /// Parse a Rhai script string into a ConfigRoot.
    pub fn parse_string(&self, script: &str) -> Result<ConfigRoot, String> {
        // Reset state for new parse
        {
            let mut state = self.state.lock();
            *state = ParserState::new();
        }

        // Run the script
        let mut scope = Scope::new();
        self.engine
            .run_with_scope(&mut scope, script)
            .map_err(|e| format!("Parse error: {}", e))?;

        // Finalize the configuration
        self.finalize_config(script)
    }

    /// Finalize the parsed configuration.
    fn finalize_config(&self, source: &str) -> Result<ConfigRoot, String> {
        let state = self.state.lock();

        // Check for unclosed device block
        if state.current_device.is_some() {
            return Err("Unclosed device_start() block - missing device_end()".to_string());
        }

        // Check for unclosed conditional blocks
        if !state.conditional_stack.is_empty() {
            return Err("Unclosed when_start() block - missing when_end()".to_string());
        }

        // Calculate SHA256 hash of source script
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        let hash_result = hasher.finalize();
        let source_hash = format!("{:x}", hash_result);

        // Get current timestamp (0 for WASM since we don't have reliable time)
        #[cfg(target_arch = "wasm32")]
        let compilation_timestamp = 0u64;
        #[cfg(not(target_arch = "wasm32"))]
        let compilation_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let metadata = Metadata {
            compilation_timestamp,
            compiler_version: "keyrx-core-0.1.0".to_string(),
            source_hash,
        };

        Ok(ConfigRoot {
            version: Version::current(),
            devices: state.devices.clone(),
            metadata,
        })
    }

    /// Validate a Rhai script without returning the configuration.
    /// Returns a list of validation errors (empty if valid).
    pub fn validate(&self, script: &str) -> Vec<error::ParseError> {
        match self.parse_string(script) {
            Ok(_) => Vec::new(),
            Err(msg) => alloc::vec![error::ParseError::Other(msg)],
        }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}
