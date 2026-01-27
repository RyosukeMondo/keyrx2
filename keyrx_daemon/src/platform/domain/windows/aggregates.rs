//! Windows-specific domain aggregates for Platform domain

#![cfg(target_os = "windows")]

use super::value_objects::{ScanCodeVO, VirtualKeyCodeVO};
use crate::platform::domain::DomainError;

/// Raw input device aggregate root
///
/// Encapsulates a Windows Raw Input device for capturing keyboard events.
/// This is an aggregate because it maintains invariants across the device's
/// input capture lifecycle.
pub struct RawInputDeviceAggregate {
    /// Device name/identifier
    name: String,
    /// Whether the device is registered
    registered: bool,
    /// Whether to capture in background (RIDEV_INPUTSINK)
    background_capture: bool,
    /// Version counter for optimistic locking
    version: u64,
}

impl RawInputDeviceAggregate {
    /// Creates a new RawInputDevice aggregate
    pub fn new(name: String) -> Self {
        Self {
            name,
            registered: false,
            background_capture: false,
            version: 0,
        }
    }

    /// Gets the device name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Checks if the device is registered
    pub fn is_registered(&self) -> bool {
        self.registered
    }

    /// Checks if background capture is enabled
    pub fn is_background_capture(&self) -> bool {
        self.background_capture
    }

    /// Gets the version for optimistic locking
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Registers the device for raw input
    pub fn register(&mut self, background_capture: bool) -> Result<(), DomainError> {
        if self.registered {
            return Err(DomainError::ConstraintViolation(
                "Device already registered".into(),
            ));
        }

        self.registered = true;
        self.background_capture = background_capture;
        self.version += 1;
        Ok(())
    }

    /// Unregisters the device
    pub fn unregister(&mut self) -> Result<(), DomainError> {
        if !self.registered {
            return Err(DomainError::ConstraintViolation(
                "Device not registered".into(),
            ));
        }

        self.registered = false;
        self.background_capture = false;
        self.version += 1;
        Ok(())
    }

    /// Validates the device state
    pub fn validate(&self) -> Result<(), DomainError> {
        // Must have a non-empty name
        if self.name.is_empty() {
            return Err(DomainError::ConstraintViolation(
                "Device name cannot be empty".into(),
            ));
        }

        Ok(())
    }
}

/// Hook callback aggregate root
///
/// Encapsulates a Windows low-level keyboard hook for capturing and suppressing events.
/// This is an aggregate because it maintains invariants across the hook's lifecycle.
pub struct HookCallbackAggregate {
    /// Hook handle (HHOOK)
    handle: Option<isize>,
    /// Whether the hook is installed
    installed: bool,
    /// Whether to suppress original events
    suppress_events: bool,
    /// Version counter for optimistic locking
    version: u64,
}

impl HookCallbackAggregate {
    /// Creates a new HookCallback aggregate
    pub fn new() -> Self {
        Self {
            handle: None,
            installed: false,
            suppress_events: true,
            version: 0,
        }
    }

    /// Gets the hook handle
    pub fn handle(&self) -> Option<isize> {
        self.handle
    }

    /// Checks if the hook is installed
    pub fn is_installed(&self) -> bool {
        self.installed
    }

    /// Checks if events are suppressed
    pub fn is_suppressing_events(&self) -> bool {
        self.suppress_events
    }

    /// Gets the version for optimistic locking
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Installs the hook
    pub fn install(&mut self, handle: isize, suppress: bool) -> Result<(), DomainError> {
        if self.installed {
            return Err(DomainError::ConstraintViolation(
                "Hook already installed".into(),
            ));
        }

        if handle == 0 {
            return Err(DomainError::InvalidDeviceHandle(
                "Hook handle cannot be null".into(),
            ));
        }

        self.handle = Some(handle);
        self.installed = true;
        self.suppress_events = suppress;
        self.version += 1;
        Ok(())
    }

    /// Uninstalls the hook
    pub fn uninstall(&mut self) -> Result<(), DomainError> {
        if !self.installed {
            return Err(DomainError::ConstraintViolation(
                "Hook not installed".into(),
            ));
        }

        self.handle = None;
        self.installed = false;
        self.version += 1;
        Ok(())
    }

    /// Enables event suppression
    pub fn enable_suppression(&mut self) -> Result<(), DomainError> {
        if !self.installed {
            return Err(DomainError::ConstraintViolation(
                "Hook not installed".into(),
            ));
        }

        if self.suppress_events {
            return Err(DomainError::ConstraintViolation(
                "Suppression already enabled".into(),
            ));
        }

        self.suppress_events = true;
        self.version += 1;
        Ok(())
    }

    /// Disables event suppression
    pub fn disable_suppression(&mut self) -> Result<(), DomainError> {
        if !self.installed {
            return Err(DomainError::ConstraintViolation(
                "Hook not installed".into(),
            ));
        }

        if !self.suppress_events {
            return Err(DomainError::ConstraintViolation(
                "Suppression already disabled".into(),
            ));
        }

        self.suppress_events = false;
        self.version += 1;
        Ok(())
    }

    /// Validates the hook state
    pub fn validate(&self) -> Result<(), DomainError> {
        // If installed, must have a handle
        if self.installed && self.handle.is_none() {
            return Err(DomainError::ConstraintViolation(
                "Installed hook must have a handle".into(),
            ));
        }

        // Handle must be non-null if present
        if let Some(handle) = self.handle {
            if handle == 0 {
                return Err(DomainError::ConstraintViolation(
                    "Hook handle cannot be null".into(),
                ));
            }
        }

        Ok(())
    }
}

impl Default for HookCallbackAggregate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_input_device_aggregate_creation() {
        let device = RawInputDeviceAggregate::new("Keyboard".into());

        assert_eq!(device.name(), "Keyboard");
        assert!(!device.is_registered());
        assert!(!device.is_background_capture());
        assert_eq!(device.version(), 0);
    }

    #[test]
    fn test_raw_input_device_aggregate_register() {
        let mut device = RawInputDeviceAggregate::new("Keyboard".into());

        device.register(true).unwrap();

        assert!(device.is_registered());
        assert!(device.is_background_capture());
        assert_eq!(device.version(), 1);
    }

    #[test]
    fn test_raw_input_device_aggregate_unregister() {
        let mut device = RawInputDeviceAggregate::new("Keyboard".into());

        device.register(true).unwrap();
        device.unregister().unwrap();

        assert!(!device.is_registered());
        assert!(!device.is_background_capture());
        assert_eq!(device.version(), 2);
    }

    #[test]
    fn test_raw_input_device_aggregate_double_register() {
        let mut device = RawInputDeviceAggregate::new("Keyboard".into());

        device.register(false).unwrap();

        // Second registration should fail
        let result = device.register(false);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));
    }

    #[test]
    fn test_hook_callback_aggregate_creation() {
        let hook = HookCallbackAggregate::new();

        assert!(!hook.is_installed());
        assert!(hook.is_suppressing_events());
        assert_eq!(hook.handle(), None);
        assert_eq!(hook.version(), 0);
    }

    #[test]
    fn test_hook_callback_aggregate_install() {
        let mut hook = HookCallbackAggregate::new();

        hook.install(0x12345678, true).unwrap();

        assert!(hook.is_installed());
        assert!(hook.is_suppressing_events());
        assert_eq!(hook.handle(), Some(0x12345678));
        assert_eq!(hook.version(), 1);
    }

    #[test]
    fn test_hook_callback_aggregate_uninstall() {
        let mut hook = HookCallbackAggregate::new();

        hook.install(0x12345678, true).unwrap();
        hook.uninstall().unwrap();

        assert!(!hook.is_installed());
        assert_eq!(hook.handle(), None);
        assert_eq!(hook.version(), 2);
    }

    #[test]
    fn test_hook_callback_aggregate_invalid_handle() {
        let mut hook = HookCallbackAggregate::new();

        // Install with null handle should fail
        let result = hook.install(0, true);
        assert!(matches!(
            result,
            Err(DomainError::InvalidDeviceHandle(_))
        ));
    }

    #[test]
    fn test_hook_callback_aggregate_suppression() {
        let mut hook = HookCallbackAggregate::new();

        hook.install(0x12345678, true).unwrap();

        // Try to disable suppression
        hook.disable_suppression().unwrap();
        assert!(!hook.is_suppressing_events());
        assert_eq!(hook.version(), 2);

        // Re-enable suppression
        hook.enable_suppression().unwrap();
        assert!(hook.is_suppressing_events());
        assert_eq!(hook.version(), 3);
    }

    #[test]
    fn test_hook_callback_aggregate_suppression_without_install() {
        let mut hook = HookCallbackAggregate::new();

        // Cannot change suppression without installing
        let result = hook.disable_suppression();
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));
    }

    #[test]
    fn test_hook_callback_aggregate_validation() {
        let mut hook = HookCallbackAggregate::new();

        // Valid uninstalled state
        assert!(hook.validate().is_ok());

        // Install
        hook.install(0x12345678, true).unwrap();
        assert!(hook.validate().is_ok());
    }
}
