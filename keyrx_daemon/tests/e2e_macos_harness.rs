//! E2E Test Harness for macOS.
//!
//! This module provides infrastructure for running end-to-end tests on macOS
//! without requiring virtual input devices (which are unavailable on macOS).
//!
//! # Components
//!
//! - [`MacosE2EError`]: Error types for macOS E2E test operations
//! - [`MacosE2EConfig`]: Test configuration with helper constructors
//! - [`MacosE2EHarness`]: Complete test orchestration
//!
//! # Example
//!
//! ```ignore
//! use keyrx_daemon::tests::e2e_macos_harness::{MacosE2EConfig, MacosE2EHarness};
//!
//! // Create a simple remap configuration
//! let config = MacosE2EConfig::simple_remap(KeyCode::A, KeyCode::B);
//!
//! // Setup the test environment (starts daemon as subprocess)
//! let harness = MacosE2EHarness::setup(config)?;
//! ```

#![cfg(target_os = "macos")]

use std::fmt;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

use keyrx_compiler::serialize::serialize as serialize_config;
use keyrx_core::config::{
    BaseKeyMapping, Condition, ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping,
    Metadata, Version,
};

// ============================================================================
// MacosE2EError - Error types for macOS E2E test operations
// ============================================================================

/// Errors that can occur during macOS E2E test operations.
#[derive(Debug)]
pub enum MacosE2EError {
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

impl std::error::Error for MacosE2EError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MacosE2EError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for MacosE2EError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MacosE2EError::ConfigError { message } => write!(f, "config error: {}", message),
            MacosE2EError::DaemonStartError { message, stderr } => {
                write!(f, "daemon start error: {}", message)?;
                if let Some(stderr) = stderr {
                    write!(f, "\nstderr: {}", stderr)?;
                }
                Ok(())
            }
            MacosE2EError::DaemonCrashed { exit_code, stderr } => {
                write!(f, "daemon crashed")?;
                if let Some(code) = exit_code {
                    write!(f, " with exit code {}", code)?;
                }
                if let Some(stderr) = stderr {
                    write!(f, "\nstderr: {}", stderr)?;
                }
                Ok(())
            }
            MacosE2EError::Timeout {
                operation,
                timeout_ms,
            } => {
                write!(
                    f,
                    "timeout after {}ms waiting for {}",
                    timeout_ms, operation
                )
            }
            MacosE2EError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl From<std::io::Error> for MacosE2EError {
    fn from(err: std::io::Error) -> Self {
        MacosE2EError::Io(err)
    }
}

// ============================================================================
// MacosE2EConfig - Test configuration with helper constructors
// ============================================================================

/// Configuration for a macOS E2E test scenario.
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
/// let config = MacosE2EConfig::simple_remap(KeyCode::A, KeyCode::B);
///
/// // Navigation layer with modifier
/// let config = MacosE2EConfig::with_modifier_layer(
///     KeyCode::CapsLock,
///     0,
///     vec![
///         (KeyCode::H, KeyCode::Left),
///         (KeyCode::J, KeyCode::Down),
///     ],
/// );
/// ```
#[derive(Debug, Clone)]
pub struct MacosE2EConfig {
    /// Device pattern for matching (default: "*" for all devices)
    pub device_pattern: String,
    /// Key mappings to apply
    pub mappings: Vec<KeyMapping>,
}

#[allow(dead_code)]
impl MacosE2EConfig {
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
    /// let config = MacosE2EConfig::simple_remap(KeyCode::CapsLock, KeyCode::Escape);
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
    /// let config = MacosE2EConfig::simple_remaps(vec![
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
    /// let config = MacosE2EConfig::modifier(KeyCode::CapsLock, 0);
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
    /// let config = MacosE2EConfig::lock(KeyCode::ScrollLock, 0);
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
    /// let config = MacosE2EConfig::conditional(0, KeyCode::H, KeyCode::Left);
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
    /// * `layer_mappings` - The key mappings active when the layer is engaged
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = MacosE2EConfig::with_modifier_layer(
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

        let base_mappings: Vec<BaseKeyMapping> = layer_mappings
            .into_iter()
            .map(|(from, to)| BaseKeyMapping::Simple { from, to })
            .collect();

        mappings.push(KeyMapping::conditional(
            Condition::ModifierActive(modifier_id),
            base_mappings,
        ));

        Self {
            device_pattern: "*".to_string(),
            mappings,
        }
    }

    /// Converts this E2E config to a full ConfigRoot for serialization.
    fn to_config_root(&self) -> ConfigRoot {
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
                compiler_version: "e2e-test-macos".to_string(),
                source_hash: "e2e-test-macos".to_string(),
            },
        }
    }
}

// ============================================================================
// TeardownResult - Results from test teardown
// ============================================================================

/// Results from tearing down a test harness.
#[derive(Debug)]
pub struct TeardownResult {
    /// Whether the daemon was sent SIGTERM
    pub sigterm_sent: bool,
    /// Whether the daemon was sent SIGKILL (after timeout)
    pub sigkill_sent: bool,
    /// Whether the daemon exited gracefully (within timeout)
    pub graceful_shutdown: bool,
    /// Daemon's exit code
    pub exit_code: Option<i32>,
}

// ============================================================================
// MacosE2EHarness - Main test harness
// ============================================================================

/// Complete test harness for macOS E2E tests.
///
/// Unlike the Linux/Windows harness, this version does not create virtual
/// input devices (macOS has no uinput equivalent). Instead, it focuses on:
///
/// - Daemon lifecycle testing (start, config load, graceful shutdown)
/// - Permission checking and graceful skipping
/// - Config compilation and loading
///
/// Full input/output testing requires manual verification or real hardware.
///
/// # Example
///
/// ```ignore
/// use keyrx_daemon::tests::e2e_macos_harness::{MacosE2EConfig, MacosE2EHarness};
///
/// let config = MacosE2EConfig::simple_remap(KeyCode::A, KeyCode::B);
/// let harness = MacosE2EHarness::setup(config)?;
///
/// // Verify daemon is running
/// assert!(harness.daemon_is_running()?);
///
/// // Graceful teardown
/// harness.teardown()?;
/// ```
pub struct MacosE2EHarness {
    /// Daemon subprocess handle.
    daemon_process: Option<Child>,
    /// Path to the temporary .krx config file.
    config_path: PathBuf,
}

/// Default timeout for daemon startup (ms)
const DAEMON_STARTUP_TIMEOUT: Duration = Duration::from_millis(2000);

/// Timeout for graceful shutdown (SIGTERM)
const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

#[allow(dead_code)]
impl MacosE2EHarness {
    /// Sets up a complete macOS E2E test environment.
    ///
    /// This method performs the following steps:
    /// 1. Generates a .krx config file
    /// 2. Starts the daemon as a subprocess with the config
    /// 3. Waits for the daemon to initialize
    ///
    /// # Arguments
    ///
    /// * `config` - Test configuration with mappings to apply
    ///
    /// # Returns
    ///
    /// Returns `MacosE2EHarness` on success, or `MacosE2EError` if:
    /// - Config serialization fails
    /// - Daemon binary not found
    /// - Daemon fails to start
    /// - Daemon exits immediately (likely permission error)
    pub fn setup(config: MacosE2EConfig) -> Result<Self, MacosE2EError> {
        Self::setup_with_timeout(config, DAEMON_STARTUP_TIMEOUT)
    }

    /// Sets up E2E environment with custom daemon startup timeout.
    ///
    /// # Arguments
    ///
    /// * `config` - Test configuration
    /// * `daemon_timeout` - How long to wait for daemon to start
    pub fn setup_with_timeout(
        config: MacosE2EConfig,
        daemon_timeout: Duration,
    ) -> Result<Self, MacosE2EError> {
        // Step 1: Generate .krx config file
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        let config_root = config.to_config_root();
        let config_bytes = serialize_config(&config_root).map_err(|e| {
            MacosE2EError::ConfigError {
                message: format!("failed to serialize config: {}", e),
            }
        })?;

        // Write to temporary file
        let config_path = std::env::temp_dir().join(format!("keyrx-e2e-macos-{}.krx", timestamp));
        let mut file = File::create(&config_path)?;
        file.write_all(&config_bytes)?;
        file.sync_all()?;

        // Step 2: Start daemon as subprocess
        let daemon_binary = Self::find_daemon_binary()?;

        let mut daemon_process = Command::new(&daemon_binary)
            .arg("run")
            .arg("--config")
            .arg(&config_path)
            .arg("--debug")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| MacosE2EError::DaemonStartError {
                message: format!("failed to spawn daemon: {}", e),
                stderr: None,
            })?;

        // Step 3: Wait for daemon to initialize
        let start = Instant::now();
        std::thread::sleep(Duration::from_millis(500));

        // Check if daemon is still running
        if let Some(status) = daemon_process.try_wait().map_err(MacosE2EError::Io)? {
            // Daemon exited immediately - capture stderr for diagnostics
            let stderr = Self::read_child_stderr(&mut daemon_process);
            return Err(MacosE2EError::DaemonCrashed {
                exit_code: status.code(),
                stderr,
            });
        }

        // Verify we're within timeout
        if start.elapsed() > daemon_timeout {
            return Err(MacosE2EError::Timeout {
                operation: "daemon initialization".to_string(),
                timeout_ms: daemon_timeout.as_millis() as u64,
            });
        }

        Ok(Self {
            daemon_process: Some(daemon_process),
            config_path,
        })
    }

    /// Checks if the daemon process is still running.
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if daemon is running, `Ok(false)` if exited,
    /// or `Err` if the check fails.
    pub fn daemon_is_running(&mut self) -> Result<bool, MacosE2EError> {
        if let Some(ref mut process) = self.daemon_process {
            match process.try_wait() {
                Ok(None) => Ok(true),
                Ok(Some(_)) => Ok(false),
                Err(e) => Err(MacosE2EError::Io(e)),
            }
        } else {
            Ok(false)
        }
    }

    /// Tears down the test environment gracefully.
    ///
    /// This method:
    /// 1. Sends SIGTERM to daemon for graceful shutdown
    /// 2. Waits up to 5 seconds
    /// 3. Sends SIGKILL if daemon doesn't exit
    /// 4. Removes temporary config file
    ///
    /// # Returns
    ///
    /// Returns [`TeardownResult`] with information about the shutdown process.
    pub fn teardown(mut self) -> Result<TeardownResult, MacosE2EError> {
        let mut result = TeardownResult {
            sigterm_sent: false,
            sigkill_sent: false,
            graceful_shutdown: false,
            exit_code: None,
        };

        if let Some(mut process) = self.daemon_process.take() {
            // Send SIGTERM for graceful shutdown
            Self::send_sigterm(&process)?;
            result.sigterm_sent = true;

            // Wait for graceful shutdown
            let start = Instant::now();
            let poll_interval = Duration::from_millis(50);

            loop {
                match process.try_wait() {
                    Ok(Some(status)) => {
                        result.graceful_shutdown = true;
                        result.exit_code = status.code();
                        break;
                    }
                    Ok(None) => {
                        if start.elapsed() >= GRACEFUL_SHUTDOWN_TIMEOUT {
                            // Timeout - force kill
                            Self::send_sigkill(&process)?;
                            result.sigkill_sent = true;

                            // Wait for forced termination
                            if let Ok(status) = process.wait() {
                                result.exit_code = status.code();
                            }
                            break;
                        }
                        std::thread::sleep(poll_interval);
                    }
                    Err(e) => return Err(MacosE2EError::Io(e)),
                }
            }
        }

        // Clean up config file
        if self.config_path.exists() {
            fs::remove_file(&self.config_path)?;
        }

        Ok(result)
    }

    /// Finds the daemon binary in the target directory.
    fn find_daemon_binary() -> Result<PathBuf, MacosE2EError> {
        // First try the explicit path from CARGO_BIN_EXE_keyrx_daemon
        if let Ok(path) = std::env::var("CARGO_BIN_EXE_keyrx_daemon") {
            let binary = PathBuf::from(path);
            if binary.exists() {
                return Ok(binary);
            }
        }

        // Fall back to searching target directories
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        let workspace_root = manifest_dir
            .parent()
            .ok_or_else(|| MacosE2EError::DaemonStartError {
                message: "failed to find workspace root".to_string(),
                stderr: None,
            })?;

        // Try both debug and release builds
        for build_type in &["debug", "release"] {
            let binary = workspace_root
                .join("target")
                .join(build_type)
                .join("keyrx_daemon");
            if binary.exists() {
                return Ok(binary);
            }
        }

        Err(MacosE2EError::DaemonStartError {
            message: "daemon binary not found in target/debug or target/release".to_string(),
            stderr: Some("Run 'cargo build -p keyrx_daemon' first".to_string()),
        })
    }

    /// Reads stderr from a child process.
    fn read_child_stderr(process: &mut Child) -> Option<String> {
        process
            .stderr
            .as_mut()
            .and_then(|stderr| std::io::read_to_string(stderr).ok())
    }

    /// Sends SIGTERM to a process for graceful shutdown.
    fn send_sigterm(process: &Child) -> Result<(), MacosE2EError> {
        use std::process::Command as StdCommand;
        StdCommand::new("kill")
            .arg("-TERM")
            .arg(process.id().to_string())
            .output()
            .map_err(MacosE2EError::Io)?;
        Ok(())
    }

    /// Sends SIGKILL to a process for forced termination.
    fn send_sigkill(process: &Child) -> Result<(), MacosE2EError> {
        use std::process::Command as StdCommand;
        StdCommand::new("kill")
            .arg("-KILL")
            .arg(process.id().to_string())
            .output()
            .map_err(MacosE2EError::Io)?;
        Ok(())
    }
}

impl Drop for MacosE2EHarness {
    /// Ensures cleanup even on panic.
    ///
    /// This method performs best-effort cleanup that never panics:
    /// 1. Terminates the daemon process (SIGTERM, then SIGKILL if needed)
    /// 2. Removes the temporary config file
    ///
    /// For explicit cleanup with error reporting, use [`teardown`](Self::teardown).
    fn drop(&mut self) {
        // Terminate daemon process
        if let Some(mut process) = self.daemon_process.take() {
            // Try graceful shutdown first with SIGTERM
            let _ = Self::send_sigterm(&process);

            // Wait briefly for graceful shutdown (short timeout in Drop)
            let start = Instant::now();
            let drop_timeout = Duration::from_millis(500);
            let poll_interval = Duration::from_millis(25);

            loop {
                match process.try_wait() {
                    Ok(Some(_)) => break, // Process exited
                    Ok(None) => {
                        if start.elapsed() >= drop_timeout {
                            // Timeout - force kill
                            let _ = Self::send_sigkill(&process);
                            // Wait for forced termination
                            let _ = process.wait();
                            break;
                        }
                        std::thread::sleep(poll_interval);
                    }
                    Err(_) => break,
                }
            }
        }

        // Clean up config file (best effort)
        if self.config_path.exists() {
            let _ = fs::remove_file(&self.config_path);
        }
    }
}
