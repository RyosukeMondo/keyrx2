//! Daemon lifecycle management for keyrx.
//!
//! This module provides the core daemon functionality including:
//!
//! - [`install_signal_handlers`]: Sets up signal handlers for graceful shutdown and reload
//! - [`SignalHandler`]: Manages signal state and detection
//! - Future: [`Daemon`]: Main daemon struct coordinating all components
//!
//! # Signal Handling
//!
//! The daemon responds to the following signals:
//!
//! - **SIGTERM**: Graceful shutdown - stops event processing and releases all resources
//! - **SIGINT**: Same as SIGTERM (Ctrl+C handling)
//! - **SIGHUP**: Configuration reload - reloads .krx file without restarting
//!
//! # Example
//!
//! ```no_run
//! use std::sync::atomic::{AtomicBool, Ordering};
//! use std::sync::Arc;
//! use keyrx_daemon::daemon::install_signal_handlers;
//!
//! let running = Arc::new(AtomicBool::new(true));
//! install_signal_handlers(running.clone()).expect("Failed to install signal handlers");
//!
//! // Main event loop
//! while running.load(Ordering::SeqCst) {
//!     // Process events
//!     std::thread::sleep(std::time::Duration::from_millis(10));
//! }
//! println!("Shutdown requested");
//! ```

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;

#[cfg(feature = "linux")]
mod linux;

#[cfg(feature = "linux")]
pub use linux::{install_signal_handlers, SignalHandler};

/// Errors that can occur during daemon operations.
#[derive(Debug, Error)]
pub enum DaemonError {
    /// Failed to install signal handlers.
    #[error("failed to install signal handlers: {0}")]
    SignalError(#[from] io::Error),

    /// Configuration error (file not found, parse error).
    #[error("configuration error: {0}")]
    ConfigError(String),

    /// Permission error (cannot grab device, cannot create uinput).
    #[error("permission error: {0}")]
    PermissionError(String),

    /// Runtime error during event processing.
    #[error("runtime error: {0}")]
    RuntimeError(String),

    /// Device discovery failed.
    #[error("device discovery failed: {0}")]
    DiscoveryError(#[from] crate::device_manager::DiscoveryError),
}

/// Exit codes for daemon termination.
///
/// These codes follow Unix conventions and are documented in the requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitCode {
    /// Successful termination.
    Success = 0,
    /// Configuration error (file not found, parse error).
    ConfigError = 1,
    /// Permission error (cannot grab device, cannot create uinput).
    PermissionError = 2,
    /// Runtime error (device disconnected with no fallback).
    RuntimeError = 3,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code as i32
    }
}

/// Reload request state.
///
/// This struct tracks whether a configuration reload has been requested
/// (typically via SIGHUP signal).
#[derive(Debug, Clone)]
pub struct ReloadState {
    /// Flag indicating a reload has been requested.
    reload_requested: Arc<AtomicBool>,
}

impl ReloadState {
    /// Creates a new reload state.
    pub fn new() -> Self {
        Self {
            reload_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns the underlying atomic flag for signal handler registration.
    pub fn flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.reload_requested)
    }

    /// Checks if a reload has been requested and clears the flag.
    ///
    /// Returns `true` if a reload was requested since the last check.
    pub fn check_and_clear(&self) -> bool {
        self.reload_requested.swap(false, Ordering::SeqCst)
    }

    /// Requests a reload (for testing purposes).
    #[cfg(test)]
    pub fn request_reload(&self) {
        self.reload_requested.store(true, Ordering::SeqCst);
    }
}

impl Default for ReloadState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_error_display() {
        let err = DaemonError::ConfigError("test error".to_string());
        assert_eq!(err.to_string(), "configuration error: test error");

        let err = DaemonError::PermissionError("access denied".to_string());
        assert_eq!(err.to_string(), "permission error: access denied");

        let err = DaemonError::RuntimeError("event loop failed".to_string());
        assert_eq!(err.to_string(), "runtime error: event loop failed");
    }

    #[test]
    fn test_exit_code_values() {
        assert_eq!(ExitCode::Success as u8, 0);
        assert_eq!(ExitCode::ConfigError as u8, 1);
        assert_eq!(ExitCode::PermissionError as u8, 2);
        assert_eq!(ExitCode::RuntimeError as u8, 3);
    }

    #[test]
    fn test_exit_code_to_i32() {
        assert_eq!(i32::from(ExitCode::Success), 0);
        assert_eq!(i32::from(ExitCode::ConfigError), 1);
        assert_eq!(i32::from(ExitCode::PermissionError), 2);
        assert_eq!(i32::from(ExitCode::RuntimeError), 3);
    }

    #[test]
    fn test_reload_state_new() {
        let state = ReloadState::new();
        assert!(!state.check_and_clear());
    }

    #[test]
    fn test_reload_state_check_and_clear() {
        let state = ReloadState::new();

        // Initially no reload requested
        assert!(!state.check_and_clear());

        // Request reload
        state.request_reload();

        // Check and clear should return true once
        assert!(state.check_and_clear());

        // Subsequent checks should return false
        assert!(!state.check_and_clear());
    }

    #[test]
    fn test_reload_state_flag_sharing() {
        let state = ReloadState::new();
        let flag = state.flag();

        // Set flag via external reference
        flag.store(true, Ordering::SeqCst);

        // Should be detectable via check_and_clear
        assert!(state.check_and_clear());
    }

    #[test]
    fn test_reload_state_default() {
        let state = ReloadState::default();
        assert!(!state.check_and_clear());
    }
}
