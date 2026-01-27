//! Linux-specific domain services for Platform domain

#![cfg(target_os = "linux")]

use super::aggregates::{EvdevDeviceAggregate, UinputDeviceAggregate};
use super::value_objects::EventCodeVO;
use crate::platform::domain::DomainError;

/// Evdev capture service
///
/// Domain service for capturing input events from evdev devices.
/// Encapsulates the business logic for event capture without depending on
/// infrastructure details.
pub struct EvdevCaptureService;

impl EvdevCaptureService {
    /// Creates a new EvdevCaptureService
    pub fn new() -> Self {
        Self
    }

    /// Validates that a device is ready for event capture
    pub fn can_capture(&self, device: &EvdevDeviceAggregate) -> Result<(), DomainError> {
        if !device.is_initialized() {
            return Err(DomainError::ConstraintViolation(
                "Device not initialized".into(),
            ));
        }

        if !device.is_grabbed() {
            return Err(DomainError::ConstraintViolation(
                "Device not grabbed".into(),
            ));
        }

        device.validate()
    }

    /// Validates an event code for capture
    pub fn validate_event_code(&self, code: EventCodeVO) -> Result<(), DomainError> {
        if !code.is_key_event() {
            return Err(DomainError::ConstraintViolation(
                "Only key events are supported".into(),
            ));
        }

        Ok(())
    }
}

impl Default for EvdevCaptureService {
    fn default() -> Self {
        Self::new()
    }
}

/// Uinput injection service
///
/// Domain service for injecting output events to uinput devices.
/// Encapsulates the business logic for event injection without depending on
/// infrastructure details.
pub struct UinputInjectionService;

impl UinputInjectionService {
    /// Creates a new UinputInjectionService
    pub fn new() -> Self {
        Self
    }

    /// Validates that a device is ready for event injection
    pub fn can_inject(&self, device: &UinputDeviceAggregate) -> Result<(), DomainError> {
        if !device.is_initialized() {
            return Err(DomainError::ConstraintViolation(
                "Device not initialized".into(),
            ));
        }

        if !device.is_created() {
            return Err(DomainError::ConstraintViolation(
                "Device not created".into(),
            ));
        }

        device.validate()
    }

    /// Validates an event code for injection
    pub fn validate_event_code(&self, code: EventCodeVO) -> Result<(), DomainError> {
        if !code.is_key_event() && !code.is_sync_event() {
            return Err(DomainError::ConstraintViolation(
                "Only key and sync events are supported".into(),
            ));
        }

        Ok(())
    }

    /// Validates an event sequence (must end with sync event)
    pub fn validate_event_sequence(&self, codes: &[EventCodeVO]) -> Result<(), DomainError> {
        if codes.is_empty() {
            return Err(DomainError::ConstraintViolation(
                "Event sequence cannot be empty".into(),
            ));
        }

        // Last event should be a sync event
        if let Some(last) = codes.last() {
            if !last.is_sync_event() {
                return Err(DomainError::ConstraintViolation(
                    "Event sequence must end with sync event".into(),
                ));
            }
        }

        Ok(())
    }
}

impl Default for UinputInjectionService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::domain::common::DevicePathVO;
    use crate::platform::domain::linux::DeviceFdVO;

    #[test]
    fn test_evdev_capture_service_can_capture() {
        let service = EvdevCaptureService::new();

        let path = DevicePathVO::new("/dev/input/event0".into()).unwrap();
        let mut device = EvdevDeviceAggregate::new(path, "Test Keyboard".into());

        // Not initialized
        let result = service.can_capture(&device);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));

        // Initialize but not grabbed
        let fd = DeviceFdVO::new(3).unwrap();
        device.open(fd).unwrap();
        let result = service.can_capture(&device);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));

        // Grabbed - should succeed
        device.grab().unwrap();
        assert!(service.can_capture(&device).is_ok());
    }

    #[test]
    fn test_evdev_capture_service_validate_event_code() {
        let service = EvdevCaptureService::new();

        let key_event = EventCodeVO::new(30); // KEY_A
        assert!(service.validate_event_code(key_event).is_ok());

        let non_key_event = EventCodeVO::new(0x0400); // Not a key event
        let result = service.validate_event_code(non_key_event);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));
    }

    #[test]
    fn test_uinput_injection_service_can_inject() {
        let service = UinputInjectionService::new();

        let mut device = UinputDeviceAggregate::new("Virtual Keyboard".into());

        // Not initialized
        let result = service.can_inject(&device);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));

        // Initialize but not created
        let fd = DeviceFdVO::new(4).unwrap();
        device.open(fd).unwrap();
        let result = service.can_inject(&device);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));

        // Created - should succeed
        device.create().unwrap();
        assert!(service.can_inject(&device).is_ok());
    }

    #[test]
    fn test_uinput_injection_service_validate_event_code() {
        let service = UinputInjectionService::new();

        let key_event = EventCodeVO::new(30); // KEY_A
        assert!(service.validate_event_code(key_event).is_ok());

        let sync_event = EventCodeVO::new(0); // EV_SYN
        assert!(service.validate_event_code(sync_event).is_ok());

        let invalid_event = EventCodeVO::new(0x0400); // Not key or sync
        let result = service.validate_event_code(invalid_event);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));
    }

    #[test]
    fn test_uinput_injection_service_validate_event_sequence() {

        let service = UinputInjectionService::new();

        // Valid sequence (ends with sync)
        let valid_seq = vec![
            EventCodeVO::new(30), // KEY_A press
            EventCodeVO::new(30), // KEY_A release
            EventCodeVO::new(0),  // EV_SYN
        ];
        assert!(service.validate_event_sequence(&valid_seq).is_ok());

        // Invalid sequence (doesn't end with sync)
        let invalid_seq = vec![
            EventCodeVO::new(30), // KEY_A press
            EventCodeVO::new(30), // KEY_A release
        ];
        let result = service.validate_event_sequence(&invalid_seq);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));

        // Empty sequence
        let empty_seq = vec![];
        let result = service.validate_event_sequence(&empty_seq);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));
    }
}
