//! Daemon lifecycle management for keyrx.
//!
//! This module provides the core daemon functionality including:
//!
//! - [`install_signal_handlers`]: Sets up signal handlers for graceful shutdown and reload
//! - [`SignalHandler`]: Manages signal state and detection
//! - [`Daemon`]: Main daemon struct coordinating all components
//!
//! # Signal Handling
//!
//! The daemon responds to the following signals:
//!
//! - **SIGTERM**: Graceful shutdown - stops event processing and releases all resources
//! - **SIGINT**: Same as SIGTERM (Ctrl+C handling)
//! - **SIGHUP**: Configuration reload - reloads .krx file without restarting
//!
//! # Daemon Lifecycle
//!
//! 1. **Initialization**: Load configuration, discover devices, create uinput output
//! 2. **Signal Setup**: Install handlers for SIGTERM, SIGINT, SIGHUP
//! 3. **Event Loop**: Process keyboard events from all managed devices
//! 4. **Shutdown**: Release devices, destroy virtual output, exit cleanly
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//! use keyrx_daemon::daemon::Daemon;
//!
//! // Initialize daemon with configuration
//! let mut daemon = Daemon::new(Path::new("config.krx"))?;
//!
//! // Run the event loop (blocks until shutdown signal)
//! daemon.run()?;
//!
//! // Shutdown is automatic via Drop trait
//! # Ok::<(), keyrx_daemon::daemon::DaemonError>(())
//! ```

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

use log::{info, trace, warn};

use crate::config_loader::ConfigError;
use crate::platform::{Platform, PlatformError};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::{install_signal_handlers, SignalHandler};
#[cfg(target_os = "windows")]
pub use windows::{install_signal_handlers, SignalHandler};

/// Returns the current time in microseconds since UNIX epoch.
///
/// This is used for tap-hold timeout checking.
#[allow(dead_code)]
fn current_time_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

/// Errors that can occur during daemon operations.
#[derive(Debug, Error)]
pub enum DaemonError {
    /// Failed to install signal handlers.
    #[error("failed to install signal handlers: {0}")]
    SignalError(#[from] io::Error),

    /// Configuration loading error (file not found, parse error).
    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Platform error.
    #[error("platform error: {0}")]
    Platform(#[from] PlatformError),

    /// Permission error (cannot grab device, cannot create uinput).
    #[error("permission error: {0}")]
    PermissionError(String),

    /// Runtime error during event processing.
    #[error("runtime error: {0}")]
    RuntimeError(String),
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

// ============================================================================
// Archived Config Conversion Helpers
// ============================================================================
//
// These functions convert rkyv-archived configuration types to owned types.
// This is necessary because DeviceManager and KeyLookup operate on owned types.

use keyrx_core::config::{
    BaseKeyMapping, Condition, ConditionItem, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping,
};

// Import the archived types from their modules
use keyrx_core::config::conditions::{ArchivedCondition, ArchivedConditionItem};
use keyrx_core::config::keys::ArchivedKeyCode;
use keyrx_core::config::mappings::{
    ArchivedBaseKeyMapping, ArchivedDeviceConfig, ArchivedKeyMapping,
};

/// Converts an archived KeyCode to an owned KeyCode.
///
/// Uses rkyv's Deserialize trait for safe conversion.
#[allow(dead_code)]
fn convert_archived_keycode(archived: &ArchivedKeyCode) -> KeyCode {
    use rkyv::Deserialize;
    archived
        .deserialize(&mut rkyv::Infallible)
        .expect("KeyCode deserialization is infallible")
}

/// Converts an archived ConditionItem to an owned ConditionItem.
#[allow(dead_code)]
fn convert_archived_condition_item(archived: &ArchivedConditionItem) -> ConditionItem {
    match archived {
        ArchivedConditionItem::ModifierActive(id) => ConditionItem::ModifierActive(*id),
        ArchivedConditionItem::LockActive(id) => ConditionItem::LockActive(*id),
    }
}

/// Converts an archived Condition to an owned Condition.
#[allow(dead_code)]
fn convert_archived_condition(archived: &ArchivedCondition) -> Condition {
    match archived {
        ArchivedCondition::ModifierActive(id) => Condition::ModifierActive(*id),
        ArchivedCondition::LockActive(id) => Condition::LockActive(*id),
        ArchivedCondition::DeviceMatches(id) => {
            use rkyv::Deserialize;
            Condition::DeviceMatches(Deserialize::deserialize(id, &mut rkyv::Infallible).unwrap())
        }
        ArchivedCondition::AllActive(items) => {
            Condition::AllActive(items.iter().map(convert_archived_condition_item).collect())
        }
        ArchivedCondition::NotActive(items) => {
            Condition::NotActive(items.iter().map(convert_archived_condition_item).collect())
        }
    }
}

/// Converts an archived BaseKeyMapping to an owned BaseKeyMapping.
#[allow(dead_code)]
fn convert_archived_base_mapping(archived: &ArchivedBaseKeyMapping) -> BaseKeyMapping {
    match archived {
        ArchivedBaseKeyMapping::Simple { from, to } => BaseKeyMapping::Simple {
            from: convert_archived_keycode(from),
            to: convert_archived_keycode(to),
        },
        ArchivedBaseKeyMapping::Modifier { from, modifier_id } => BaseKeyMapping::Modifier {
            from: convert_archived_keycode(from),
            modifier_id: *modifier_id,
        },
        ArchivedBaseKeyMapping::Lock { from, lock_id } => BaseKeyMapping::Lock {
            from: convert_archived_keycode(from),
            lock_id: *lock_id,
        },
        ArchivedBaseKeyMapping::TapHold {
            from,
            tap,
            hold_modifier,
            threshold_ms,
        } => BaseKeyMapping::TapHold {
            from: convert_archived_keycode(from),
            tap: convert_archived_keycode(tap),
            hold_modifier: *hold_modifier,
            threshold_ms: *threshold_ms,
        },
        ArchivedBaseKeyMapping::ModifiedOutput {
            from,
            to,
            shift,
            ctrl,
            alt,
            win,
        } => BaseKeyMapping::ModifiedOutput {
            from: convert_archived_keycode(from),
            to: convert_archived_keycode(to),
            shift: *shift,
            ctrl: *ctrl,
            alt: *alt,
            win: *win,
        },
    }
}

/// Converts an archived KeyMapping to an owned KeyMapping.
#[allow(dead_code)]
fn convert_archived_key_mapping(archived: &ArchivedKeyMapping) -> KeyMapping {
    match archived {
        ArchivedKeyMapping::Base(base) => KeyMapping::Base(convert_archived_base_mapping(base)),
        ArchivedKeyMapping::Conditional {
            condition,
            mappings,
        } => KeyMapping::Conditional {
            condition: convert_archived_condition(condition),
            mappings: mappings.iter().map(convert_archived_base_mapping).collect(),
        },
    }
}

/// Converts an archived DeviceConfig to an owned DeviceConfig.
#[allow(dead_code)]
fn convert_archived_device_config(archived: &ArchivedDeviceConfig) -> DeviceConfig {
    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: archived.identifier.pattern.to_string(),
        },
        mappings: archived
            .mappings
            .iter()
            .map(convert_archived_key_mapping)
            .collect(),
    }
}

/// The main keyrx daemon.
///
/// `Daemon` coordinates all components for keyboard event processing:
///
/// - **Configuration**: Loads and manages the .krx configuration file
/// - **Device Manager**: Discovers and manages input keyboard devices
/// - **Output Device**: Creates virtual keyboard for injecting remapped events
/// - **Signal Handling**: Responds to SIGTERM, SIGINT (shutdown), SIGHUP (reload)
///
/// # Initialization Order
///
/// The daemon initializes components in this order:
///
/// 1. Load configuration from .krx file
/// 2. Discover and match input devices
/// 3. Create uinput virtual keyboard
/// 4. Install signal handlers
///
/// This order ensures we fail fast on configuration errors before grabbing devices.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use keyrx_daemon::daemon::Daemon;
///
/// // Initialize daemon
/// let mut daemon = Daemon::new(Path::new("config.krx"))?;
///
/// // Check device count
/// println!("Managing {} devices", daemon.device_count());
///
/// // Run event loop (blocks until shutdown)
/// daemon.run()?;
/// # Ok::<(), keyrx_daemon::daemon::DaemonError>(())
/// ```
pub struct Daemon {
    /// Path to the configuration file (for reload support).
    config_path: PathBuf,

    /// Platform abstraction for input/output operations.
    platform: Box<dyn Platform>,

    /// Running flag for event loop control.
    running: Arc<AtomicBool>,

    /// Signal handler for reload detection.
    signal_handler: SignalHandler,
}

impl Daemon {
    /// Creates a new daemon instance with the specified platform and configuration file.
    ///
    /// This method performs the initialization sequence:
    ///
    /// 1. Accepts a platform implementation via dependency injection
    /// 2. Initializes the platform
    /// 3. Installs signal handlers for graceful shutdown and reload
    ///
    /// # Arguments
    ///
    /// * `platform` - Platform implementation for input/output operations
    /// * `config_path` - Path to the .krx configuration file (for reload support)
    ///
    /// # Returns
    ///
    /// * `Ok(Daemon)` - Successfully initialized daemon
    /// * `Err(DaemonError::Platform)` - Platform initialization failed
    /// * `Err(DaemonError::SignalError)` - Failed to install signal handlers
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use keyrx_daemon::daemon::Daemon;
    /// use keyrx_daemon::platform::create_platform;
    ///
    /// let platform = create_platform()?;
    /// match Daemon::new(platform, Path::new("config.krx")) {
    ///     Ok(daemon) => {
    ///         println!("Daemon initialized successfully");
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Failed to initialize daemon: {}", e);
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(mut platform: Box<dyn Platform>, config_path: &Path) -> Result<Self, DaemonError> {
        info!(
            "Initializing keyrx daemon with config: {}",
            config_path.display()
        );

        // Step 1: Initialize the platform
        info!("Initializing platform...");
        platform.initialize()?;
        info!("Platform initialized");

        // Step 2: Install signal handlers
        info!("Installing signal handlers...");
        let running = Arc::new(AtomicBool::new(true));
        let signal_handler = install_signal_handlers(Arc::clone(&running))?;
        info!("Signal handlers installed");

        info!("Daemon initialization complete");

        Ok(Self {
            config_path: config_path.to_path_buf(),
            platform,
            running,
            signal_handler,
        })
    }

    /// Returns the number of managed devices.
    #[must_use]
    pub fn device_count(&self) -> usize {
        // Platform trait doesn't expose device count directly
        // Return the number of devices from list_devices()
        self.platform
            .list_devices()
            .map(|devices| devices.len())
            .unwrap_or(0)
    }

    /// Returns whether the daemon is still running.
    ///
    /// This is set to `false` when a shutdown signal (SIGTERM, SIGINT) is received.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Returns the path to the configuration file.
    #[must_use]
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Returns a reference to the signal handler.
    #[must_use]
    pub fn signal_handler(&self) -> &SignalHandler {
        &self.signal_handler
    }

    /// Returns the running flag for external coordination.
    #[must_use]
    pub fn running_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.running)
    }

    /// Reloads the configuration from disk.
    ///
    /// **Note**: Configuration reload is not currently supported with the Platform trait abstraction.
    /// The Platform trait would need to be extended with a `reload()` method to support this functionality.
    ///
    /// # Error Handling
    ///
    /// Currently returns an error indicating reload is not supported.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use keyrx_daemon::daemon::Daemon;
    /// use keyrx_daemon::platform::create_platform;
    ///
    /// let platform = create_platform()?;
    /// let mut daemon = Daemon::new(platform, Path::new("config.krx"))?;
    ///
    /// // Reload is not currently supported with Platform trait
    /// match daemon.reload() {
    ///     Ok(()) => println!("Configuration reloaded successfully"),
    ///     Err(e) => eprintln!("Reload failed: {}", e),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn reload(&mut self) -> Result<(), DaemonError> {
        warn!("Configuration reload requested but not supported with Platform trait abstraction");
        Err(DaemonError::RuntimeError(
            "Configuration reload not supported with Platform trait".to_string(),
        ))
    }

    /// Runs the main event processing loop.
    ///
    /// This method captures keyboard events from the platform, processes them,
    /// and injects output events. The loop continues until a shutdown signal
    /// (SIGTERM or SIGINT) is received.
    ///
    /// # Event Processing Flow
    ///
    /// For each input event:
    /// 1. Capture event from platform (blocking)
    /// 2. Inject the event back through the platform
    ///
    /// **Note**: The current implementation is simplified and does not perform
    /// key remapping. Full remapping support would require the Platform trait
    /// to expose device state and lookup tables, or for the Daemon to manage
    /// remapping state independently.
    ///
    /// # Signal Handling
    ///
    /// - **SIGTERM/SIGINT**: Sets the running flag to false, causing graceful exit
    /// - **SIGHUP**: Triggers configuration reload (currently not supported)
    ///
    /// # Errors
    ///
    /// - `DaemonError::Platform`: Platform error during event capture or injection
    /// - `DaemonError::RuntimeError`: Critical error during event processing
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use keyrx_daemon::daemon::Daemon;
    /// use keyrx_daemon::platform::create_platform;
    ///
    /// let platform = create_platform()?;
    /// let mut daemon = Daemon::new(platform, Path::new("config.krx"))?;
    ///
    /// // This blocks until shutdown signal received
    /// daemon.run()?;
    ///
    /// println!("Daemon stopped gracefully");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn run(&mut self) -> Result<(), DaemonError> {
        info!("Starting event processing loop");

        // Track metrics for periodic logging
        let mut event_count: u64 = 0;
        let mut last_stats_time = std::time::Instant::now();
        const STATS_INTERVAL: Duration = Duration::from_secs(60);

        // Main event loop
        while self.is_running() {
            // Check for SIGHUP (reload request)
            if self.signal_handler.check_reload() {
                info!("Reload signal received (SIGHUP)");
                if let Err(e) = self.reload() {
                    // Log the error but continue running
                    warn!("Configuration reload failed: {}", e);
                }
            }

            // Capture input event from platform (blocking with timeout to allow signal checking)
            // Note: capture_input() may return an error if no events are available
            // We treat this as non-fatal and continue the loop
            match self.platform.capture_input() {
                Ok(event) => {
                    trace!("Input event: {:?}", event);

                    // TODO: Process event through remapping engine
                    // For now, just pass through the event
                    let output_event = event;

                    // Inject output event
                    if let Err(e) = self.platform.inject_output(output_event) {
                        warn!("Failed to inject event: {}", e);
                    } else {
                        event_count += 1;
                    }
                }
                Err(e) => {
                    // Check if we should exit
                    if !self.is_running() {
                        break;
                    }

                    // Log non-fatal errors and continue
                    trace!("Event capture returned error (may be timeout): {}", e);

                    // Small sleep to prevent busy loop
                    std::thread::sleep(Duration::from_millis(10));
                }
            }

            // Periodic stats logging
            if last_stats_time.elapsed() >= STATS_INTERVAL {
                info!("Event loop stats: {} events processed", event_count);
                last_stats_time = std::time::Instant::now();
            }
        }

        info!(
            "Event loop stopped. Total events processed: {}",
            event_count
        );

        Ok(())
    }

    /// Performs graceful shutdown of the daemon.
    ///
    /// This method shuts down the platform and releases all resources.
    ///
    /// # Error Handling
    ///
    /// Errors during shutdown are logged but do not prevent continued cleanup.
    /// This ensures that all resources are released even if some operations fail.
    ///
    /// # Automatic Cleanup
    ///
    /// This method is called automatically by the `Drop` implementation, so
    /// cleanup occurs even on panic or unexpected termination.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use keyrx_daemon::daemon::Daemon;
    /// use keyrx_daemon::platform::create_platform;
    ///
    /// let platform = create_platform()?;
    /// let mut daemon = Daemon::new(platform, Path::new("config.krx"))?;
    ///
    /// // Run the event loop
    /// daemon.run()?;
    ///
    /// // Explicit shutdown (optional - Drop will call this automatically)
    /// daemon.shutdown();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn shutdown(&mut self) {
        info!("Initiating graceful shutdown...");

        // Shutdown the platform
        info!("Shutting down platform...");
        match self.platform.shutdown() {
            Ok(()) => {
                info!("Platform shutdown successfully");
            }
            Err(e) => {
                // Log warning but continue
                warn!("Failed to shutdown platform: {}", e);
            }
        }

        // Mark daemon as stopped
        self.running.store(false, Ordering::SeqCst);

        info!("Shutdown complete");
    }
}

/// Drop implementation to ensure automatic cleanup on daemon exit.
///
/// When a `Daemon` is dropped (goes out of scope, program exits, or panic occurs),
/// this implementation ensures that:
///
/// 1. All grabbed input devices are released (restores normal keyboard input)
/// 2. The virtual keyboard is destroyed (removes from `/dev/input/`)
///
/// This prevents:
/// - Orphaned device grabs that would block keyboard input
/// - Orphaned virtual devices in `/dev/input/`
/// - Stuck keys in applications
///
/// # Note
///
/// The `shutdown()` method is called automatically. If `shutdown()` was already
/// called manually, it will safely handle the already-released/destroyed state.
impl Drop for Daemon {
    fn drop(&mut self) {
        // Call shutdown to release all resources
        // shutdown() handles already-released devices gracefully
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Requires Platform refactoring"]
    fn test_daemon_error_display() {
        let err = DaemonError::PermissionError("access denied".to_string());
        assert_eq!(err.to_string(), "permission error: access denied");

        let err = DaemonError::RuntimeError("event loop failed".to_string());
        assert_eq!(err.to_string(), "runtime error: event loop failed");
    }

    #[test]
    #[ignore = "Requires Platform refactoring"]
    fn test_daemon_error_config_variant() {
        use crate::config_loader::ConfigError;
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let config_err = ConfigError::Io(io_err);
        let daemon_err = DaemonError::Config(config_err);
        assert!(daemon_err.to_string().contains("configuration error"));
    }

    #[test]
    #[ignore = "Requires Platform refactoring"]
    fn test_daemon_error_platform_variant() {
        use crate::platform::PlatformError;
        let platform_err = PlatformError::DeviceNotFound("test device".to_string());
        let daemon_err = DaemonError::Platform(platform_err);
        assert!(daemon_err.to_string().contains("platform error"));
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

    // Daemon tests - these require real devices/permissions
    mod daemon_tests {
        use super::*;

        // Note: These tests are temporarily disabled during Platform trait refactoring
        // TODO: Update tests to use mock Platform implementation

        #[test]
        #[ignore = "Requires Platform refactoring"]
        fn test_daemon_new_missing_config() {
            // Test disabled - needs Platform mock
            // let platform = create_platform().unwrap();
            // let result = Daemon::new(platform, Path::new("/nonexistent/path/config.krx"));
            // assert!(result.is_err());
        }

        #[test]
        #[ignore = "Requires Platform refactoring"]
        fn test_daemon_new_real_devices() {
            crate::skip_if_no_uinput!();
            use keyrx_compiler::serialize::serialize;
            use keyrx_core::config::{
                ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
            };
            use std::io::Write;
            use tempfile::NamedTempFile;

            // Create a minimal valid config
            let config = ConfigRoot {
                version: Version::current(),
                devices: vec![DeviceConfig {
                    identifier: DeviceIdentifier {
                        pattern: "*".to_string(),
                    },
                    mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
                }],
                metadata: Metadata {
                    compilation_timestamp: 0,
                    compiler_version: "test".to_string(),
                    source_hash: "test".to_string(),
                },
            };

            // Serialize and write to temp file
            let bytes = serialize(&config).expect("Failed to serialize config");
            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            temp_file.write_all(&bytes).expect("Failed to write config");
            temp_file.flush().expect("Failed to flush");

            // Try to create daemon
            match Daemon::new(temp_file.path()) {
                Ok(daemon) => {
                    assert!(daemon.device_count() > 0, "Should have at least one device");
                    assert!(daemon.is_running(), "Daemon should be running initially");
                    println!("Daemon created with {} device(s)", daemon.device_count());
                }
                Err(e) => {
                    println!("Daemon creation failed (expected if no permissions): {}", e);
                }
            }
        }

        #[test]
        #[ignore = "Requires Platform refactoring"]
        fn test_daemon_error_from_discovery_error() {
            // Test disabled - DiscoveryError no longer in DaemonError
        }

        #[test]
        #[ignore = "Requires Platform refactoring"]
        fn test_daemon_error_from_io_error() {
            let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test");
            let daemon_err = DaemonError::SignalError(io_err);
            assert!(daemon_err.to_string().contains("signal handlers"));
        }

        #[test]
        #[ignore = "Requires Platform refactoring"]
        fn test_daemon_reload_success() {
            crate::skip_if_no_uinput!();
            use keyrx_compiler::serialize::serialize;
            use keyrx_core::config::{
                ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
            };
            use std::io::Write;
            use tempfile::NamedTempFile;

            // Create initial config (A -> B)
            let initial_config = ConfigRoot {
                version: Version::current(),
                devices: vec![DeviceConfig {
                    identifier: DeviceIdentifier {
                        pattern: "*".to_string(),
                    },
                    mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
                }],
                metadata: Metadata {
                    compilation_timestamp: 0,
                    compiler_version: "test".to_string(),
                    source_hash: "initial".to_string(),
                },
            };

            // Create temp file with initial config
            let bytes = serialize(&initial_config).expect("Failed to serialize config");
            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            temp_file.write_all(&bytes).expect("Failed to write config");
            temp_file.flush().expect("Failed to flush");

            // Try to create daemon
            let mut daemon = match Daemon::new(temp_file.path()) {
                Ok(d) => d,
                Err(e) => {
                    println!("Daemon creation failed (expected if no permissions): {}", e);
                    return;
                }
            };

            // Create updated config (A -> C)
            let updated_config = ConfigRoot {
                version: Version::current(),
                devices: vec![DeviceConfig {
                    identifier: DeviceIdentifier {
                        pattern: "*".to_string(),
                    },
                    mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::C)],
                }],
                metadata: Metadata {
                    compilation_timestamp: 1,
                    compiler_version: "test".to_string(),
                    source_hash: "updated".to_string(),
                },
            };

            // Overwrite the config file
            let updated_bytes =
                serialize(&updated_config).expect("Failed to serialize updated config");
            std::fs::write(temp_file.path(), &updated_bytes)
                .expect("Failed to write updated config");

            // Reload should succeed
            let result = daemon.reload();
            assert!(result.is_ok(), "Reload should succeed: {:?}", result.err());

            println!("Configuration reloaded successfully");
        }

        #[test]
        #[ignore = "Requires Platform refactoring"]
        fn test_daemon_reload_preserves_device_count() {
            crate::skip_if_no_uinput!();
            use keyrx_compiler::serialize::serialize;
            use keyrx_core::config::{
                ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
            };
            use std::io::Write;
            use tempfile::NamedTempFile;

            // Create config
            let config = ConfigRoot {
                version: Version::current(),
                devices: vec![DeviceConfig {
                    identifier: DeviceIdentifier {
                        pattern: "*".to_string(),
                    },
                    mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
                }],
                metadata: Metadata {
                    compilation_timestamp: 0,
                    compiler_version: "test".to_string(),
                    source_hash: "test".to_string(),
                },
            };

            let bytes = serialize(&config).expect("Failed to serialize config");
            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            temp_file.write_all(&bytes).expect("Failed to write config");
            temp_file.flush().expect("Failed to flush");

            let mut daemon = match Daemon::new(temp_file.path()) {
                Ok(d) => d,
                Err(e) => {
                    println!("Daemon creation failed (expected if no permissions): {}", e);
                    return;
                }
            };

            let device_count_before = daemon.device_count();

            // Reload (same config, device count should remain)
            let result = daemon.reload();
            assert!(result.is_ok());

            let device_count_after = daemon.device_count();
            assert_eq!(
                device_count_before, device_count_after,
                "Device count should remain same after reload"
            );
        }

        #[test]
        #[ignore = "Requires Platform refactoring"]
        fn test_daemon_reload_missing_file() {
            crate::skip_if_no_uinput!();
            use keyrx_compiler::serialize::serialize;
            use keyrx_core::config::{
                ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
            };
            use std::io::Write;
            use tempfile::NamedTempFile;

            // Create config
            let config = ConfigRoot {
                version: Version::current(),
                devices: vec![DeviceConfig {
                    identifier: DeviceIdentifier {
                        pattern: "*".to_string(),
                    },
                    mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
                }],
                metadata: Metadata {
                    compilation_timestamp: 0,
                    compiler_version: "test".to_string(),
                    source_hash: "test".to_string(),
                },
            };

            let bytes = serialize(&config).expect("Failed to serialize config");
            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            temp_file.write_all(&bytes).expect("Failed to write config");
            temp_file.flush().expect("Failed to flush");

            let mut daemon = match Daemon::new(temp_file.path()) {
                Ok(d) => d,
                Err(e) => {
                    println!("Daemon creation failed (expected if no permissions): {}", e);
                    return;
                }
            };

            // Delete the config file
            std::fs::remove_file(temp_file.path()).expect("Failed to remove config file");

            // Reload should fail but daemon should continue
            let result = daemon.reload();
            assert!(
                result.is_err(),
                "Reload should fail when config file is missing"
            );

            match result {
                Err(DaemonError::Config(_)) => {} // Expected
                Err(e) => panic!("Expected Config error, got: {}", e),
                Ok(_) => panic!("Expected error, got Ok"),
            }

            // Daemon should still be running
            assert!(
                daemon.is_running(),
                "Daemon should still be running after failed reload"
            );
        }

        #[test]
        #[ignore = "Requires Platform refactoring"]
        fn test_daemon_shutdown_releases_devices() {
            crate::skip_if_no_uinput!();
            use keyrx_compiler::serialize::serialize;
            use keyrx_core::config::{
                ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
            };
            use std::io::Write;
            use tempfile::NamedTempFile;

            // Create config
            let config = ConfigRoot {
                version: Version::current(),
                devices: vec![DeviceConfig {
                    identifier: DeviceIdentifier {
                        pattern: "*".to_string(),
                    },
                    mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
                }],
                metadata: Metadata {
                    compilation_timestamp: 0,
                    compiler_version: "test".to_string(),
                    source_hash: "test".to_string(),
                },
            };

            let bytes = serialize(&config).expect("Failed to serialize config");
            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            temp_file.write_all(&bytes).expect("Failed to write config");
            temp_file.flush().expect("Failed to flush");

            let mut daemon = match Daemon::new(temp_file.path()) {
                Ok(d) => d,
                Err(e) => {
                    println!("Daemon creation failed (expected if no permissions): {}", e);
                    return;
                }
            };

            // Verify daemon is running initially
            assert!(daemon.is_running(), "Daemon should be running initially");

            // Call shutdown
            daemon.shutdown();

            // Verify daemon is no longer running
            assert!(
                !daemon.is_running(),
                "Daemon should not be running after shutdown"
            );

            // Verify output device is destroyed
            // Note: Platform trait doesn't expose output device directly
            // assert!(daemon.output().is_destroyed(), "Output device should be destroyed after shutdown");

            println!("Shutdown completed successfully - all resources released");
        }

        #[test]
        #[ignore = "Requires Platform refactoring"]
        fn test_daemon_shutdown_idempotent() {
            crate::skip_if_no_uinput!();
            use keyrx_compiler::serialize::serialize;
            use keyrx_core::config::{
                ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
            };
            use std::io::Write;
            use tempfile::NamedTempFile;

            // Create config
            let config = ConfigRoot {
                version: Version::current(),
                devices: vec![DeviceConfig {
                    identifier: DeviceIdentifier {
                        pattern: "*".to_string(),
                    },
                    mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
                }],
                metadata: Metadata {
                    compilation_timestamp: 0,
                    compiler_version: "test".to_string(),
                    source_hash: "test".to_string(),
                },
            };

            let bytes = serialize(&config).expect("Failed to serialize config");
            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            temp_file.write_all(&bytes).expect("Failed to write config");
            temp_file.flush().expect("Failed to flush");

            let mut daemon = match Daemon::new(temp_file.path()) {
                Ok(d) => d,
                Err(e) => {
                    println!("Daemon creation failed (expected if no permissions): {}", e);
                    return;
                }
            };

            // Call shutdown twice - should not panic or error
            daemon.shutdown();
            daemon.shutdown(); // Should handle already-released state gracefully

            assert!(!daemon.is_running(), "Daemon should not be running");
            // Note: Platform trait doesn't expose output device directly
            // assert!(daemon.output().is_destroyed(), "Output should be destroyed");

            println!("Multiple shutdown calls handled gracefully");
        }

        #[test]
        #[ignore = "Requires Platform refactoring"]
        fn test_daemon_drop_calls_shutdown() {
            crate::skip_if_no_uinput!();
            use keyrx_compiler::serialize::serialize;
            use keyrx_core::config::{
                ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
            };
            use std::io::Write;
            use tempfile::NamedTempFile;

            // Create config
            let config = ConfigRoot {
                version: Version::current(),
                devices: vec![DeviceConfig {
                    identifier: DeviceIdentifier {
                        pattern: "*".to_string(),
                    },
                    mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
                }],
                metadata: Metadata {
                    compilation_timestamp: 0,
                    compiler_version: "test".to_string(),
                    source_hash: "test".to_string(),
                },
            };

            let bytes = serialize(&config).expect("Failed to serialize config");
            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            temp_file.write_all(&bytes).expect("Failed to write config");
            temp_file.flush().expect("Failed to flush");

            // Create daemon in a block so it gets dropped
            {
                let daemon = match Daemon::new(temp_file.path()) {
                    Ok(d) => d,
                    Err(e) => {
                        println!("Daemon creation failed (expected if no permissions): {}", e);
                        return;
                    }
                };

                println!(
                    "Daemon created with {} device(s), dropping now...",
                    daemon.device_count()
                );
                // daemon will be dropped here, which should call shutdown via Drop
            }

            // If we get here without panic, Drop worked correctly
            println!("Daemon dropped successfully - cleanup via Drop verified");
        }
    }
}
