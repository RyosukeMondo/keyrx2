//! Windows-specific domain services for Platform domain

#![cfg(target_os = "windows")]

use super::aggregates::{HookCallbackAggregate, RawInputDeviceAggregate};
use super::value_objects::{ScanCodeVO, VirtualKeyCodeVO};
use crate::platform::domain::DomainError;

/// Low-level hook service
///
/// Domain service for managing Windows low-level keyboard hooks.
/// Encapsulates the business logic for hook management without depending on
/// infrastructure details.
pub struct LowLevelHookService;

impl LowLevelHookService {
    /// Creates a new LowLevelHookService
    pub fn new() -> Self {
        Self
    }

    /// Validates that a hook is ready for capturing events
    pub fn can_capture(&self, hook: &HookCallbackAggregate) -> Result<(), DomainError> {
        if !hook.is_installed() {
            return Err(DomainError::ConstraintViolation(
                "Hook not installed".into(),
            ));
        }

        hook.validate()
    }

    /// Validates that event suppression is configured correctly
    pub fn can_suppress_events(&self, hook: &HookCallbackAggregate) -> Result<(), DomainError> {
        if !hook.is_installed() {
            return Err(DomainError::ConstraintViolation(
                "Hook not installed".into(),
            ));
        }

        if !hook.is_suppressing_events() {
            return Err(DomainError::ConstraintViolation(
                "Event suppression not enabled".into(),
            ));
        }

        hook.validate()
    }

    /// Validates a virtual key code for processing
    pub fn validate_virtual_key(&self, vk: VirtualKeyCodeVO) -> Result<(), DomainError> {
        if !vk.is_valid() {
            return Err(DomainError::ConstraintViolation(
                "Invalid virtual key code".into(),
            ));
        }

        Ok(())
    }

    /// Validates a scan code for processing
    pub fn validate_scan_code(&self, scan: ScanCodeVO) -> Result<(), DomainError> {
        if !scan.is_valid() {
            return Err(DomainError::ConstraintViolation(
                "Invalid scan code".into(),
            ));
        }

        Ok(())
    }
}

impl Default for LowLevelHookService {
    fn default() -> Self {
        Self::new()
    }
}

/// SendInput service
///
/// Domain service for injecting keyboard events via Windows SendInput API.
/// Encapsulates the business logic for event injection without depending on
/// infrastructure details.
pub struct SendInputService;

impl SendInputService {
    /// Creates a new SendInputService
    pub fn new() -> Self {
        Self
    }

    /// Validates a virtual key code for injection
    pub fn validate_virtual_key(&self, vk: VirtualKeyCodeVO) -> Result<(), DomainError> {
        if !vk.is_valid() {
            return Err(DomainError::ConstraintViolation(
                "Invalid virtual key code".into(),
            ));
        }

        Ok(())
    }

    /// Validates a scan code for injection
    pub fn validate_scan_code(&self, scan: ScanCodeVO) -> Result<(), DomainError> {
        if !scan.is_valid() {
            return Err(DomainError::ConstraintViolation(
                "Invalid scan code".into(),
            ));
        }

        Ok(())
    }

    /// Validates an event pair (press + release)
    pub fn validate_event_pair(
        &self,
        press_vk: VirtualKeyCodeVO,
        release_vk: VirtualKeyCodeVO,
    ) -> Result<(), DomainError> {
        if press_vk != release_vk {
            return Err(DomainError::ConstraintViolation(
                "Press and release must use the same key code".into(),
            ));
        }

        self.validate_virtual_key(press_vk)?;
        self.validate_virtual_key(release_vk)?;

        Ok(())
    }

    /// Validates a modifier sequence (must release in reverse order)
    pub fn validate_modifier_sequence(
        &self,
        keys: &[VirtualKeyCodeVO],
    ) -> Result<(), DomainError> {
        if keys.is_empty() {
            return Err(DomainError::ConstraintViolation(
                "Modifier sequence cannot be empty".into(),
            ));
        }

        // All keys must be valid
        for key in keys {
            self.validate_virtual_key(*key)?;
        }

        // Check if all are modifiers (optional validation)
        let all_modifiers = keys.iter().all(|k| k.is_modifier());
        if !all_modifiers {
            return Err(DomainError::ConstraintViolation(
                "All keys in modifier sequence must be modifiers".into(),
            ));
        }

        Ok(())
    }
}

impl Default for SendInputService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_level_hook_service_can_capture() {
        let service = LowLevelHookService::new();

        let mut hook = HookCallbackAggregate::new();

        // Not installed
        let result = service.can_capture(&hook);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));

        // Install
        hook.install(0x12345678, true).unwrap();
        assert!(service.can_capture(&hook).is_ok());
    }

    #[test]
    fn test_low_level_hook_service_can_suppress_events() {
        let service = LowLevelHookService::new();

        let mut hook = HookCallbackAggregate::new();

        // Not installed
        let result = service.can_suppress_events(&hook);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));

        // Install with suppression
        hook.install(0x12345678, true).unwrap();
        assert!(service.can_suppress_events(&hook).is_ok());

        // Disable suppression
        hook.disable_suppression().unwrap();
        let result = service.can_suppress_events(&hook);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));
    }

    #[test]
    fn test_low_level_hook_service_validate_virtual_key() {
        let service = LowLevelHookService::new();

        let valid_vk = VirtualKeyCodeVO::new(0x41); // VK_A
        assert!(service.validate_virtual_key(valid_vk).is_ok());

        let invalid_vk = VirtualKeyCodeVO::new(0x00);
        let result = service.validate_virtual_key(invalid_vk);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));
    }

    #[test]
    fn test_low_level_hook_service_validate_scan_code() {
        let service = LowLevelHookService::new();

        let valid_scan = ScanCodeVO::new(0x1E, false); // A key
        assert!(service.validate_scan_code(valid_scan).is_ok());

        let invalid_scan = ScanCodeVO::new(0, false);
        let result = service.validate_scan_code(invalid_scan);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));
    }

    #[test]
    fn test_send_input_service_validate_virtual_key() {
        let service = SendInputService::new();

        let valid_vk = VirtualKeyCodeVO::new(0x41); // VK_A
        assert!(service.validate_virtual_key(valid_vk).is_ok());

        let invalid_vk = VirtualKeyCodeVO::new(0x00);
        let result = service.validate_virtual_key(invalid_vk);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));
    }

    #[test]
    fn test_send_input_service_validate_event_pair() {
        let service = SendInputService::new();

        let vk_a = VirtualKeyCodeVO::new(0x41); // VK_A
        let vk_b = VirtualKeyCodeVO::new(0x42); // VK_B

        // Same key - valid
        assert!(service.validate_event_pair(vk_a, vk_a).is_ok());

        // Different keys - invalid
        let result = service.validate_event_pair(vk_a, vk_b);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));
    }

    #[test]
    fn test_send_input_service_validate_modifier_sequence() {

        let service = SendInputService::new();

        // Valid modifier sequence
        let modifiers = vec![
            VirtualKeyCodeVO::new(0x10), // VK_SHIFT
            VirtualKeyCodeVO::new(0x11), // VK_CONTROL
            VirtualKeyCodeVO::new(0x12), // VK_MENU (Alt)
        ];
        assert!(service.validate_modifier_sequence(&modifiers).is_ok());

        // Mixed modifiers and non-modifiers
        let mixed = vec![
            VirtualKeyCodeVO::new(0x10), // VK_SHIFT
            VirtualKeyCodeVO::new(0x41), // VK_A (not a modifier)
        ];
        let result = service.validate_modifier_sequence(&mixed);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));

        // Empty sequence
        let empty = vec![];
        let result = service.validate_modifier_sequence(&empty);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));
    }
}
