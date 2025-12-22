//! E2E Test Harness for Virtual Keyboard Testing.
//!
//! This module provides infrastructure for running end-to-end tests using
//! virtual input devices (uinput) instead of requiring physical hardware.
//!
//! # Components
//!
//! - [`E2EError`]: Error types for E2E test operations
//! - [`E2EConfig`]: Test configuration with helper constructors
//!
//! # Example
//!
//! ```ignore
//! use keyrx_daemon::tests::e2e_harness::{E2EConfig, E2EError};
//!
//! // Create a simple remap configuration
//! let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
//! ```

#![cfg(all(target_os = "linux", feature = "linux"))]

use std::fmt;

use keyrx_core::config::{
    BaseKeyMapping, Condition, ConditionItem, ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode,
    KeyMapping, Metadata, Version,
};
use keyrx_core::runtime::KeyEvent;
use keyrx_daemon::test_utils::VirtualDeviceError;

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
}
