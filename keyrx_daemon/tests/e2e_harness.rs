//! E2E Test Harness for Virtual Keyboard Testing.
//!
//! This module provides infrastructure for running end-to-end tests using
//! virtual input devices (uinput) instead of requiring physical hardware.
//!
//! # Components
//!
//! - [`E2EError`]: Error types for E2E test operations
//! - [`E2EConfig`]: Test configuration with helper constructors
//! - [`E2EHarness`]: Complete test orchestration
//!
//! # Example
//!
//! ```ignore
//! use keyrx_daemon::tests::e2e_harness::{E2EConfig, E2EHarness};
//!
//! // Create a simple remap configuration
//! let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
//!
//! // Setup the test environment (starts daemon as subprocess)
//! let harness = E2EHarness::setup(config)?;
//! ```

#![cfg(all(target_os = "linux", feature = "linux"))]

use std::fmt;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

use keyrx_compiler::serialize::serialize as serialize_config;
use keyrx_core::config::{
    BaseKeyMapping, Condition, ConditionItem, ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode,
    KeyMapping, Metadata, Version,
};
use keyrx_core::runtime::KeyEvent;
use keyrx_daemon::test_utils::{OutputCapture, VirtualDeviceError, VirtualKeyboard};

// ============================================================================
// E2EError - Error types for E2E test operations
// ============================================================================

/// Errors that can occur during E2E test operations.
///
/// This error type wraps [`VirtualDeviceError`] and adds E2E-specific error
/// variants for test setup, execution, and verification.
#[derive(Debug)]
pub enum E2EError {
    /// Error from virtual device operations (VirtualKeyboard, OutputCapture).
    VirtualDevice(VirtualDeviceError),

    /// Failed to create or serialize test configuration.
    ConfigError {
        /// Description of what went wrong
        message: String,
    },

    /// Failed to start daemon subprocess.
    DaemonStartError {
        /// Description of what went wrong
        message: String,
        /// Standard error output from daemon, if available
        stderr: Option<String>,
    },

    /// Daemon exited unexpectedly during test.
    DaemonCrashed {
        /// Exit code if available
        exit_code: Option<i32>,
        /// Standard error output from daemon, if available
        stderr: Option<String>,
    },

    /// Test verification failed - captured events don't match expected.
    VerificationFailed {
        /// Events that were captured during the test
        captured: Vec<KeyEvent>,
        /// Events that were expected
        expected: Vec<KeyEvent>,
        /// Detailed diff message
        diff: String,
    },

    /// Test timed out waiting for expected condition.
    Timeout {
        /// What operation timed out
        operation: String,
        /// How long we waited
        timeout_ms: u64,
    },

    /// I/O error during test operations.
    Io(std::io::Error),
}

impl std::error::Error for E2EError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            E2EError::VirtualDevice(e) => Some(e),
            E2EError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for E2EError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            E2EError::VirtualDevice(e) => write!(f, "virtual device error: {}", e),
            E2EError::ConfigError { message } => write!(f, "config error: {}", message),
            E2EError::DaemonStartError { message, stderr } => {
                write!(f, "daemon start error: {}", message)?;
                if let Some(stderr) = stderr {
                    write!(f, "\nstderr: {}", stderr)?;
                }
                Ok(())
            }
            E2EError::DaemonCrashed { exit_code, stderr } => {
                write!(f, "daemon crashed")?;
                if let Some(code) = exit_code {
                    write!(f, " with exit code {}", code)?;
                }
                if let Some(stderr) = stderr {
                    write!(f, "\nstderr: {}", stderr)?;
                }
                Ok(())
            }
            E2EError::VerificationFailed {
                captured,
                expected,
                diff,
            } => {
                writeln!(f, "verification failed:")?;
                writeln!(f, "  expected {} event(s): {:?}", expected.len(), expected)?;
                writeln!(f, "  captured {} event(s): {:?}", captured.len(), captured)?;
                write!(f, "\n{}", diff)
            }
            E2EError::Timeout {
                operation,
                timeout_ms,
            } => {
                write!(
                    f,
                    "timeout after {}ms waiting for {}",
                    timeout_ms, operation
                )
            }
            E2EError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl From<VirtualDeviceError> for E2EError {
    fn from(err: VirtualDeviceError) -> Self {
        E2EError::VirtualDevice(err)
    }
}

impl From<std::io::Error> for E2EError {
    fn from(err: std::io::Error) -> Self {
        E2EError::Io(err)
    }
}

// ============================================================================
// E2EConfig - Test configuration with helper constructors
// ============================================================================

/// Configuration for an E2E test scenario.
///
/// Provides helper constructors to easily create test configurations for
/// common remapping scenarios. The configuration includes:
///
/// - Device pattern for matching keyboards
/// - Key mappings to apply
///
/// # Example
///
/// ```ignore
/// // Simple A → B remapping
/// let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
///
/// // Navigation layer with modifier
/// let config = E2EConfig::with_modifier_layer(
///     KeyCode::CapsLock,
///     0,
///     vec![
///         (KeyCode::H, KeyCode::Left),
///         (KeyCode::J, KeyCode::Down),
///     ],
/// );
/// ```
#[derive(Debug, Clone)]
pub struct E2EConfig {
    /// Device pattern for matching (default: "*" for all devices)
    pub device_pattern: String,
    /// Key mappings to apply
    pub mappings: Vec<KeyMapping>,
}

impl E2EConfig {
    /// Creates a new E2EConfig with the given device pattern and mappings.
    pub fn new(device_pattern: impl Into<String>, mappings: Vec<KeyMapping>) -> Self {
        Self {
            device_pattern: device_pattern.into(),
            mappings,
        }
    }

    /// Creates a configuration with a simple key remapping (A → B).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::simple_remap(KeyCode::CapsLock, KeyCode::Escape);
    /// ```
    pub fn simple_remap(from: KeyCode, to: KeyCode) -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: vec![KeyMapping::simple(from, to)],
        }
    }

    /// Creates a configuration with multiple simple remappings.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::simple_remaps(vec![
    ///     (KeyCode::A, KeyCode::B),
    ///     (KeyCode::CapsLock, KeyCode::Escape),
    /// ]);
    /// ```
    pub fn simple_remaps(remaps: Vec<(KeyCode, KeyCode)>) -> Self {
        let mappings = remaps
            .into_iter()
            .map(|(from, to)| KeyMapping::simple(from, to))
            .collect();

        Self {
            device_pattern: "*".to_string(),
            mappings,
        }
    }

    /// Creates a configuration with a custom modifier key.
    ///
    /// The modifier key will set internal state when held, but produces no
    /// output events.
    ///
    /// # Arguments
    ///
    /// * `from` - The key that activates the modifier
    /// * `modifier_id` - The modifier ID (0-254)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::modifier(KeyCode::CapsLock, 0);
    /// ```
    pub fn modifier(from: KeyCode, modifier_id: u8) -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: vec![KeyMapping::modifier(from, modifier_id)],
        }
    }

    /// Creates a configuration with a toggle lock key.
    ///
    /// The lock key toggles internal state on press (no output on release).
    ///
    /// # Arguments
    ///
    /// * `from` - The key that toggles the lock
    /// * `lock_id` - The lock ID (0-254)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::lock(KeyCode::ScrollLock, 0);
    /// ```
    pub fn lock(from: KeyCode, lock_id: u8) -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: vec![KeyMapping::lock(from, lock_id)],
        }
    }

    /// Creates a configuration with a conditional mapping.
    ///
    /// Maps `from` → `to` only when the specified modifier is active.
    ///
    /// # Arguments
    ///
    /// * `modifier_id` - The modifier that must be active
    /// * `from` - Source key for the mapping
    /// * `to` - Target key for the mapping
    ///
    /// # Example
    ///
    /// ```ignore
    /// // When modifier 0 is active, H → Left
    /// let config = E2EConfig::conditional(0, KeyCode::H, KeyCode::Left);
    /// ```
    pub fn conditional(modifier_id: u8, from: KeyCode, to: KeyCode) -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: vec![KeyMapping::conditional(
                Condition::ModifierActive(modifier_id),
                vec![BaseKeyMapping::Simple { from, to }],
            )],
        }
    }

    /// Creates a configuration with a modifier key and conditional mappings.
    ///
    /// This is the common pattern for navigation layers (e.g., Vim-style HJKL).
    ///
    /// # Arguments
    ///
    /// * `modifier_key` - The key that activates the layer
    /// * `modifier_id` - The modifier ID for the layer
    /// * `layer_mappings` - List of (from, to) pairs active when layer is held
    ///
    /// # Example
    ///
    /// ```ignore
    /// // CapsLock activates layer, HJKL become arrow keys
    /// let config = E2EConfig::with_modifier_layer(
    ///     KeyCode::CapsLock,
    ///     0,
    ///     vec![
    ///         (KeyCode::H, KeyCode::Left),
    ///         (KeyCode::J, KeyCode::Down),
    ///         (KeyCode::K, KeyCode::Up),
    ///         (KeyCode::L, KeyCode::Right),
    ///     ],
    /// );
    /// ```
    pub fn with_modifier_layer(
        modifier_key: KeyCode,
        modifier_id: u8,
        layer_mappings: Vec<(KeyCode, KeyCode)>,
    ) -> Self {
        let mut mappings = vec![KeyMapping::modifier(modifier_key, modifier_id)];

        for (from, to) in layer_mappings {
            mappings.push(KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(modifier_id)]),
                vec![BaseKeyMapping::Simple { from, to }],
            ));
        }

        Self {
            device_pattern: "*".to_string(),
            mappings,
        }
    }

    /// Creates a configuration with a lock key and conditional mappings.
    ///
    /// Similar to modifier layer but uses toggle lock instead of momentary hold.
    ///
    /// # Arguments
    ///
    /// * `lock_key` - The key that toggles the layer
    /// * `lock_id` - The lock ID for the layer
    /// * `layer_mappings` - List of (from, to) pairs active when lock is on
    ///
    /// # Example
    ///
    /// ```ignore
    /// // ScrollLock toggles layer, number row becomes F-keys
    /// let config = E2EConfig::with_lock_layer(
    ///     KeyCode::ScrollLock,
    ///     0,
    ///     vec![
    ///         (KeyCode::Num1, KeyCode::F1),
    ///         (KeyCode::Num2, KeyCode::F2),
    ///     ],
    /// );
    /// ```
    pub fn with_lock_layer(
        lock_key: KeyCode,
        lock_id: u8,
        layer_mappings: Vec<(KeyCode, KeyCode)>,
    ) -> Self {
        let mut mappings = vec![KeyMapping::lock(lock_key, lock_id)];

        for (from, to) in layer_mappings {
            mappings.push(KeyMapping::conditional(
                Condition::LockActive(lock_id),
                vec![BaseKeyMapping::Simple { from, to }],
            ));
        }

        Self {
            device_pattern: "*".to_string(),
            mappings,
        }
    }

    /// Creates a configuration with a modified output mapping.
    ///
    /// When `from` is pressed, outputs `to` with specified physical modifiers.
    ///
    /// # Arguments
    ///
    /// * `from` - Source key
    /// * `to` - Target key
    /// * `shift` - Include Shift modifier
    /// * `ctrl` - Include Ctrl modifier
    /// * `alt` - Include Alt modifier
    /// * `win` - Include Win/Meta modifier
    ///
    /// # Example
    ///
    /// ```ignore
    /// // A → Shift+1 (outputs '!')
    /// let config = E2EConfig::modified_output(
    ///     KeyCode::A,
    ///     KeyCode::Num1,
    ///     true, false, false, false,
    /// );
    /// ```
    pub fn modified_output(
        from: KeyCode,
        to: KeyCode,
        shift: bool,
        ctrl: bool,
        alt: bool,
        win: bool,
    ) -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: vec![KeyMapping::modified_output(from, to, shift, ctrl, alt, win)],
        }
    }

    /// Adds additional mappings to this configuration.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B)
    ///     .with_mappings(vec![
    ///         KeyMapping::simple(KeyCode::C, KeyCode::D),
    ///     ]);
    /// ```
    pub fn with_mappings(mut self, mappings: Vec<KeyMapping>) -> Self {
        self.mappings.extend(mappings);
        self
    }

    /// Sets the device pattern for this configuration.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B)
    ///     .with_device_pattern("USB*");
    /// ```
    pub fn with_device_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.device_pattern = pattern.into();
        self
    }

    /// Converts this E2EConfig to a ConfigRoot for serialization.
    ///
    /// This creates a complete configuration with proper version and metadata.
    pub fn to_config_root(&self) -> ConfigRoot {
        ConfigRoot {
            version: Version::current(),
            devices: vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: self.device_pattern.clone(),
                },
                mappings: self.mappings.clone(),
            }],
            metadata: Metadata {
                compilation_timestamp: 0,
                compiler_version: "e2e-test".to_string(),
                source_hash: "e2e-test".to_string(),
            },
        }
    }
}

impl Default for E2EConfig {
    fn default() -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: Vec::new(),
        }
    }
}

// ============================================================================
// E2EHarness - Complete E2E test orchestration
// ============================================================================

/// Default name for the daemon's virtual output device.
const DAEMON_OUTPUT_NAME: &str = "keyrx Virtual Keyboard";

/// Default timeout for waiting for daemon to be ready.
const DAEMON_STARTUP_TIMEOUT: Duration = Duration::from_secs(5);

/// Default timeout for waiting for output device to appear.
const OUTPUT_DEVICE_TIMEOUT: Duration = Duration::from_secs(5);

/// Orchestrates complete E2E test lifecycle.
///
/// This harness manages:
/// - Creation of a virtual input keyboard
/// - Generation and writing of test configuration
/// - Starting the daemon as a subprocess
/// - Finding and connecting to the daemon's output device
/// - Cleanup of all resources on drop
///
/// # Example
///
/// ```ignore
/// use keyrx_daemon::tests::e2e_harness::{E2EConfig, E2EHarness};
/// use keyrx_core::config::KeyCode;
///
/// // Create a simple A→B remapping test
/// let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
/// let harness = E2EHarness::setup(config)?;
///
/// // Harness is now ready for testing
/// // - Virtual keyboard created
/// // - Daemon running and grabbing the virtual keyboard
/// // - Output capture connected to daemon's output
/// ```
pub struct E2EHarness {
    /// Virtual keyboard for injecting test input events.
    virtual_input: VirtualKeyboard,
    /// Daemon subprocess handle.
    daemon_process: Option<Child>,
    /// Output capture for reading daemon's remapped events.
    output_capture: OutputCapture,
    /// Path to the temporary .krx config file.
    config_path: PathBuf,
    /// Captured stderr from daemon for diagnostics.
    /// This field will be populated and used by future teardown implementation.
    #[allow(dead_code)]
    daemon_stderr: Option<String>,
}

impl E2EHarness {
    /// Sets up a complete E2E test environment.
    ///
    /// This method performs the following steps:
    /// 1. Creates a VirtualKeyboard with a unique name
    /// 2. Generates a .krx config file targeting the virtual keyboard
    /// 3. Starts the daemon as a subprocess with the config
    /// 4. Waits for the daemon to grab the device and create its output
    /// 5. Finds and opens the daemon's output device
    ///
    /// # Arguments
    ///
    /// * `config` - Test configuration with mappings to apply
    ///
    /// # Returns
    ///
    /// An `E2EHarness` ready for test input/output operations.
    ///
    /// # Errors
    ///
    /// - [`E2EError::VirtualDevice`] if virtual keyboard creation fails
    /// - [`E2EError::ConfigError`] if config serialization fails
    /// - [`E2EError::DaemonStartError`] if daemon fails to start
    /// - [`E2EError::Timeout`] if daemon doesn't become ready in time
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::simple_remap(KeyCode::CapsLock, KeyCode::Escape);
    /// let harness = E2EHarness::setup(config)?;
    /// ```
    pub fn setup(config: E2EConfig) -> Result<Self, E2EError> {
        Self::setup_with_timeout(config, DAEMON_STARTUP_TIMEOUT, OUTPUT_DEVICE_TIMEOUT)
    }

    /// Sets up E2E environment with custom timeouts.
    ///
    /// # Arguments
    ///
    /// * `config` - Test configuration
    /// * `daemon_timeout` - How long to wait for daemon to start
    /// * `output_timeout` - How long to wait for output device
    pub fn setup_with_timeout(
        config: E2EConfig,
        _daemon_timeout: Duration,
        output_timeout: Duration,
    ) -> Result<Self, E2EError> {
        // Step 1: Create virtual keyboard with unique name
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let input_name = format!("e2e-test-input-{}", timestamp);

        let virtual_input = VirtualKeyboard::create(&input_name)?;

        // Give the kernel a moment to register the device
        std::thread::sleep(Duration::from_millis(100));

        // Step 2: Generate .krx config file targeting the virtual keyboard
        // Modify the config to match our virtual keyboard's name
        let test_config = E2EConfig {
            device_pattern: virtual_input.name().to_string(),
            mappings: config.mappings,
        };

        let config_root = test_config.to_config_root();
        let config_bytes = serialize_config(&config_root).map_err(|e| E2EError::ConfigError {
            message: format!("failed to serialize config: {}", e),
        })?;

        // Write to temporary file
        let config_path = std::env::temp_dir().join(format!("keyrx-e2e-{}.krx", timestamp));
        let mut file = File::create(&config_path)?;
        file.write_all(&config_bytes)?;
        file.sync_all()?;

        // Step 3: Start daemon as subprocess
        let daemon_binary = Self::find_daemon_binary()?;

        let mut daemon_process = Command::new(&daemon_binary)
            .arg("run")
            .arg("--config")
            .arg(&config_path)
            .arg("--debug")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| E2EError::DaemonStartError {
                message: format!("failed to spawn daemon: {}", e),
                stderr: None,
            })?;

        // Step 4: Wait for daemon to start and grab our device
        // We do this by waiting for the output device to appear
        let start = Instant::now();

        // Give daemon a moment to initialize
        std::thread::sleep(Duration::from_millis(200));

        // Check if daemon is still running
        if let Some(status) = daemon_process.try_wait().map_err(E2EError::Io)? {
            // Daemon exited immediately - capture stderr for diagnostics
            let stderr = Self::read_child_stderr(&mut daemon_process);
            return Err(E2EError::DaemonCrashed {
                exit_code: status.code(),
                stderr,
            });
        }

        // Step 5: Find and open the daemon's output device
        let remaining_timeout = output_timeout.saturating_sub(start.elapsed());
        let mut output_capture = OutputCapture::find_by_name(DAEMON_OUTPUT_NAME, remaining_timeout)
            .map_err(|e| match e {
                VirtualDeviceError::NotFound { .. } | VirtualDeviceError::Timeout { .. } => {
                    // Daemon may have crashed - check and include stderr
                    let stderr = Self::read_child_stderr(&mut daemon_process);
                    if let Some(status) = daemon_process.try_wait().ok().flatten() {
                        E2EError::DaemonCrashed {
                            exit_code: status.code(),
                            stderr,
                        }
                    } else {
                        E2EError::Timeout {
                            operation: format!(
                                "waiting for output device '{}'",
                                DAEMON_OUTPUT_NAME
                            ),
                            timeout_ms: output_timeout.as_millis() as u64,
                        }
                    }
                }
                _ => E2EError::VirtualDevice(e),
            })?;

        // Drain any pending events from output capture
        let _ = output_capture.drain();

        Ok(Self {
            virtual_input,
            daemon_process: Some(daemon_process),
            output_capture,
            config_path,
            daemon_stderr: None,
        })
    }

    /// Returns a reference to the virtual input keyboard.
    #[must_use]
    pub fn virtual_input(&self) -> &VirtualKeyboard {
        &self.virtual_input
    }

    /// Returns a mutable reference to the virtual input keyboard.
    pub fn virtual_input_mut(&mut self) -> &mut VirtualKeyboard {
        &mut self.virtual_input
    }

    /// Returns a reference to the output capture.
    #[must_use]
    pub fn output_capture(&self) -> &OutputCapture {
        &self.output_capture
    }

    /// Returns a mutable reference to the output capture.
    pub fn output_capture_mut(&mut self) -> &mut OutputCapture {
        &mut self.output_capture
    }

    /// Returns the config file path.
    #[must_use]
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Returns whether the daemon process is still running.
    #[must_use]
    pub fn is_daemon_running(&mut self) -> bool {
        if let Some(ref mut process) = self.daemon_process {
            matches!(process.try_wait(), Ok(None))
        } else {
            false
        }
    }

    /// Finds the daemon binary path.
    ///
    /// Looks in the following order:
    /// 1. `target/debug/keyrx_daemon` (debug build)
    /// 2. `target/release/keyrx_daemon` (release build)
    /// 3. Path from `KEYRX_DAEMON_PATH` environment variable
    fn find_daemon_binary() -> Result<PathBuf, E2EError> {
        // Check environment variable first
        if let Ok(path) = std::env::var("KEYRX_DAEMON_PATH") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }

        // Try workspace target directory
        // We navigate from the test crate to the workspace root
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let workspace_root = PathBuf::from(&manifest_dir)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        // Try debug build first
        let debug_path = workspace_root.join("target/debug/keyrx_daemon");
        if debug_path.exists() {
            return Ok(debug_path);
        }

        // Try release build
        let release_path = workspace_root.join("target/release/keyrx_daemon");
        if release_path.exists() {
            return Ok(release_path);
        }

        Err(E2EError::ConfigError {
            message: format!(
                "Could not find keyrx_daemon binary. Tried:\n\
                 - {}\n\
                 - {}\n\
                 Set KEYRX_DAEMON_PATH environment variable to specify the path.",
                debug_path.display(),
                release_path.display()
            ),
        })
    }

    /// Reads stderr from the daemon process if available.
    fn read_child_stderr(child: &mut Child) -> Option<String> {
        use std::io::Read;
        child.stderr.as_mut().and_then(|stderr| {
            let mut buf = String::new();
            stderr.read_to_string(&mut buf).ok()?;
            if buf.is_empty() {
                None
            } else {
                Some(buf)
            }
        })
    }
}

impl Drop for E2EHarness {
    /// Ensures cleanup even on panic.
    ///
    /// This method:
    /// 1. Terminates the daemon process (SIGTERM, then SIGKILL if needed)
    /// 2. Removes the temporary config file
    /// 3. Virtual keyboard is dropped automatically
    fn drop(&mut self) {
        // Terminate daemon process
        if let Some(mut process) = self.daemon_process.take() {
            // Try graceful shutdown first with SIGTERM
            #[cfg(unix)]
            unsafe {
                libc::kill(process.id() as libc::pid_t, libc::SIGTERM);
            }

            // Wait briefly for graceful shutdown
            std::thread::sleep(Duration::from_millis(100));

            // Check if still running and force kill if needed
            if let Ok(None) = process.try_wait() {
                let _ = process.kill();
            }

            // Wait for process to fully exit
            let _ = process.wait();
        }

        // Clean up config file
        let _ = fs::remove_file(&self.config_path);
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // E2EError Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_e2e_error_display_virtual_device() {
        let vd_err = VirtualDeviceError::uinput_permission_denied();
        let err = E2EError::from(vd_err);
        let msg = err.to_string();
        assert!(msg.contains("virtual device error"));
        assert!(msg.contains("permission denied"));
    }

    #[test]
    fn test_e2e_error_display_config_error() {
        let err = E2EError::ConfigError {
            message: "invalid mapping".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("config error"));
        assert!(msg.contains("invalid mapping"));
    }

    #[test]
    fn test_e2e_error_display_daemon_start() {
        let err = E2EError::DaemonStartError {
            message: "failed to spawn".to_string(),
            stderr: Some("permission denied".to_string()),
        };
        let msg = err.to_string();
        assert!(msg.contains("daemon start error"));
        assert!(msg.contains("failed to spawn"));
        assert!(msg.contains("permission denied"));
    }

    #[test]
    fn test_e2e_error_display_daemon_crashed() {
        let err = E2EError::DaemonCrashed {
            exit_code: Some(1),
            stderr: Some("segfault".to_string()),
        };
        let msg = err.to_string();
        assert!(msg.contains("daemon crashed"));
        assert!(msg.contains("exit code 1"));
        assert!(msg.contains("segfault"));
    }

    #[test]
    fn test_e2e_error_display_verification_failed() {
        let err = E2EError::VerificationFailed {
            captured: vec![KeyEvent::Press(KeyCode::A)],
            expected: vec![KeyEvent::Press(KeyCode::B)],
            diff: "line 1: expected B, got A".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("verification failed"));
        assert!(msg.contains("expected 1 event"));
        assert!(msg.contains("captured 1 event"));
    }

    #[test]
    fn test_e2e_error_display_timeout() {
        let err = E2EError::Timeout {
            operation: "event capture".to_string(),
            timeout_ms: 5000,
        };
        let msg = err.to_string();
        assert!(msg.contains("timeout"));
        assert!(msg.contains("5000ms"));
        assert!(msg.contains("event capture"));
    }

    #[test]
    fn test_e2e_error_display_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = E2EError::from(io_err);
        let msg = err.to_string();
        assert!(msg.contains("I/O error"));
        assert!(msg.contains("file not found"));
    }

    // ------------------------------------------------------------------------
    // E2EConfig Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_e2e_config_simple_remap() {
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
        assert_eq!(config.device_pattern, "*");
        assert_eq!(config.mappings.len(), 1);

        match &config.mappings[0] {
            KeyMapping::Base(BaseKeyMapping::Simple { from, to }) => {
                assert_eq!(*from, KeyCode::A);
                assert_eq!(*to, KeyCode::B);
            }
            _ => panic!("Expected Simple mapping"),
        }
    }

    #[test]
    fn test_e2e_config_simple_remaps() {
        let config =
            E2EConfig::simple_remaps(vec![(KeyCode::A, KeyCode::B), (KeyCode::C, KeyCode::D)]);
        assert_eq!(config.mappings.len(), 2);
    }

    #[test]
    fn test_e2e_config_modifier() {
        let config = E2EConfig::modifier(KeyCode::CapsLock, 0);
        assert_eq!(config.mappings.len(), 1);

        match &config.mappings[0] {
            KeyMapping::Base(BaseKeyMapping::Modifier { from, modifier_id }) => {
                assert_eq!(*from, KeyCode::CapsLock);
                assert_eq!(*modifier_id, 0);
            }
            _ => panic!("Expected Modifier mapping"),
        }
    }

    #[test]
    fn test_e2e_config_lock() {
        let config = E2EConfig::lock(KeyCode::ScrollLock, 1);
        assert_eq!(config.mappings.len(), 1);

        match &config.mappings[0] {
            KeyMapping::Base(BaseKeyMapping::Lock { from, lock_id }) => {
                assert_eq!(*from, KeyCode::ScrollLock);
                assert_eq!(*lock_id, 1);
            }
            _ => panic!("Expected Lock mapping"),
        }
    }

    #[test]
    fn test_e2e_config_conditional() {
        let config = E2EConfig::conditional(0, KeyCode::H, KeyCode::Left);
        assert_eq!(config.mappings.len(), 1);

        match &config.mappings[0] {
            KeyMapping::Conditional {
                condition,
                mappings,
            } => {
                assert!(matches!(condition, Condition::ModifierActive(0)));
                assert_eq!(mappings.len(), 1);
                match &mappings[0] {
                    BaseKeyMapping::Simple { from, to } => {
                        assert_eq!(*from, KeyCode::H);
                        assert_eq!(*to, KeyCode::Left);
                    }
                    _ => panic!("Expected Simple base mapping"),
                }
            }
            _ => panic!("Expected Conditional mapping"),
        }
    }

    #[test]
    fn test_e2e_config_with_modifier_layer() {
        let config = E2EConfig::with_modifier_layer(
            KeyCode::CapsLock,
            0,
            vec![(KeyCode::H, KeyCode::Left), (KeyCode::J, KeyCode::Down)],
        );

        // Should have: 1 modifier + 2 conditional mappings
        assert_eq!(config.mappings.len(), 3);

        // First mapping should be the modifier
        match &config.mappings[0] {
            KeyMapping::Base(BaseKeyMapping::Modifier { from, modifier_id }) => {
                assert_eq!(*from, KeyCode::CapsLock);
                assert_eq!(*modifier_id, 0);
            }
            _ => panic!("Expected Modifier as first mapping"),
        }

        // Second and third should be conditional
        assert!(matches!(
            &config.mappings[1],
            KeyMapping::Conditional { .. }
        ));
        assert!(matches!(
            &config.mappings[2],
            KeyMapping::Conditional { .. }
        ));
    }

    #[test]
    fn test_e2e_config_with_lock_layer() {
        let config = E2EConfig::with_lock_layer(
            KeyCode::ScrollLock,
            0,
            vec![(KeyCode::Num1, KeyCode::F1), (KeyCode::Num2, KeyCode::F2)],
        );

        // Should have: 1 lock + 2 conditional mappings
        assert_eq!(config.mappings.len(), 3);

        // First mapping should be the lock
        match &config.mappings[0] {
            KeyMapping::Base(BaseKeyMapping::Lock { from, lock_id }) => {
                assert_eq!(*from, KeyCode::ScrollLock);
                assert_eq!(*lock_id, 0);
            }
            _ => panic!("Expected Lock as first mapping"),
        }
    }

    #[test]
    fn test_e2e_config_modified_output() {
        let config = E2EConfig::modified_output(
            KeyCode::A,
            KeyCode::Num1,
            true,  // shift
            false, // ctrl
            false, // alt
            false, // win
        );

        assert_eq!(config.mappings.len(), 1);

        match &config.mappings[0] {
            KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
                from,
                to,
                shift,
                ctrl,
                alt,
                win,
            }) => {
                assert_eq!(*from, KeyCode::A);
                assert_eq!(*to, KeyCode::Num1);
                assert!(*shift);
                assert!(!*ctrl);
                assert!(!*alt);
                assert!(!*win);
            }
            _ => panic!("Expected ModifiedOutput mapping"),
        }
    }

    #[test]
    fn test_e2e_config_with_mappings() {
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B)
            .with_mappings(vec![KeyMapping::simple(KeyCode::C, KeyCode::D)]);

        assert_eq!(config.mappings.len(), 2);
    }

    #[test]
    fn test_e2e_config_with_device_pattern() {
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B).with_device_pattern("USB*");

        assert_eq!(config.device_pattern, "USB*");
    }

    #[test]
    fn test_e2e_config_to_config_root() {
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
        let root = config.to_config_root();

        assert_eq!(root.version, Version::current());
        assert_eq!(root.devices.len(), 1);
        assert_eq!(root.devices[0].identifier.pattern, "*");
        assert_eq!(root.devices[0].mappings.len(), 1);
        assert_eq!(root.metadata.compiler_version, "e2e-test");
    }

    #[test]
    fn test_e2e_config_default() {
        let config = E2EConfig::default();
        assert_eq!(config.device_pattern, "*");
        assert!(config.mappings.is_empty());
    }

    // ------------------------------------------------------------------------
    // E2EHarness Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_daemon_output_name_constant() {
        // Verify the constant matches what the daemon creates
        assert_eq!(DAEMON_OUTPUT_NAME, "keyrx Virtual Keyboard");
    }

    #[test]
    fn test_daemon_startup_timeout_is_reasonable() {
        // Timeout should be between 1 and 30 seconds
        assert!(DAEMON_STARTUP_TIMEOUT >= Duration::from_secs(1));
        assert!(DAEMON_STARTUP_TIMEOUT <= Duration::from_secs(30));
    }

    #[test]
    fn test_output_device_timeout_is_reasonable() {
        // Timeout should be between 1 and 30 seconds
        assert!(OUTPUT_DEVICE_TIMEOUT >= Duration::from_secs(1));
        assert!(OUTPUT_DEVICE_TIMEOUT <= Duration::from_secs(30));
    }

    #[test]
    fn test_find_daemon_binary_uses_environment_variable() {
        // Set up a fake path via environment variable
        // This won't find the binary but tests the lookup logic
        std::env::set_var("KEYRX_DAEMON_PATH", "/nonexistent/path/keyrx_daemon");

        let result = E2EHarness::find_daemon_binary();

        // Clean up
        std::env::remove_var("KEYRX_DAEMON_PATH");

        // Should fail because the path doesn't exist
        // But it should try the environment variable first
        // Since it doesn't exist, it falls through to trying workspace paths
        // The test verifies the function exists and returns a Result
        match result {
            Ok(path) => {
                // If we found a binary, it should exist
                assert!(path.exists());
            }
            Err(E2EError::ConfigError { message }) => {
                // Expected when no binary is available
                assert!(message.contains("Could not find keyrx_daemon binary"));
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    /// Test that E2EHarness::setup creates all components correctly
    /// Note: Marked #[ignore] because it requires uinput access and daemon binary
    #[test]
    #[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux test_e2e_harness_setup -- --ignored"]
    fn test_e2e_harness_setup() {
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
        let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

        // Verify components are initialized
        assert!(!harness.virtual_input().name().is_empty());
        assert!(harness
            .virtual_input()
            .name()
            .starts_with("e2e-test-input-"));
        assert!(harness.config_path().exists());
        assert!(harness.is_daemon_running());

        // Harness will be cleaned up on drop
    }

    /// Test that E2EHarness cleanup works on drop
    /// Note: Marked #[ignore] because it requires uinput access and daemon binary
    #[test]
    #[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux test_e2e_harness_cleanup -- --ignored"]
    fn test_e2e_harness_cleanup() {
        let config_path;

        {
            let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
            let harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

            config_path = harness.config_path().clone();
            assert!(config_path.exists(), "Config file should exist during test");

            // Harness is dropped here
        }

        // After drop, config file should be cleaned up
        assert!(
            !config_path.exists(),
            "Config file should be removed after drop"
        );
    }

    /// Test that E2EHarness fails gracefully when daemon binary is missing
    #[test]
    fn test_e2e_harness_setup_fails_without_binary() {
        // Set environment to a nonexistent path to ensure binary isn't found
        let original_path = std::env::var("KEYRX_DAEMON_PATH").ok();
        std::env::set_var("KEYRX_DAEMON_PATH", "/definitely/nonexistent/keyrx_daemon");

        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);

        // Setup should fail either with VirtualDevice error (no uinput access)
        // or ConfigError (no daemon binary)
        let result = E2EHarness::setup(config);

        // Restore environment
        match original_path {
            Some(path) => std::env::set_var("KEYRX_DAEMON_PATH", path),
            None => std::env::remove_var("KEYRX_DAEMON_PATH"),
        }

        // Should be an error
        assert!(result.is_err());
    }
}
