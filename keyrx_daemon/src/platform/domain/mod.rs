//! Domain-Driven Design module for Platform domain
//!
//! This module implements DDD patterns for platform-specific input/output operations:
//! - Aggregates: PlatformDeviceAggregate, EvdevDeviceAggregate, UinputDeviceAggregate,
//!   RawInputDeviceAggregate, HookCallbackAggregate
//! - Value Objects: DevicePathVO, DeviceHandleVO, EventCodeVO, DeviceFdVO,
//!   VirtualKeyCodeVO, ScanCodeVO
//! - Domain Services: EvdevCaptureService, UinputInjectionService, LowLevelHookService,
//!   SendInputService
//! - Repositories: PlatformDeviceRepository (trait)

pub mod common;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

// Re-export common types
pub use common::{
    DeviceHandleVO, DevicePathVO, PlatformDeviceAggregate, PlatformDeviceRepository,
};

// Re-export Linux types
#[cfg(target_os = "linux")]
pub use linux::{
    DeviceFdVO, EventCodeVO, EvdevCaptureService, EvdevDeviceAggregate, UinputDeviceAggregate,
    UinputInjectionService,
};

// Re-export Windows types
#[cfg(target_os = "windows")]
pub use windows::{
    HookCallbackAggregate, LowLevelHookService, RawInputDeviceAggregate, ScanCodeVO,
    SendInputService, VirtualKeyCodeVO,
};

/// Domain error type for Platform domain
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    /// Invalid device path
    InvalidDevicePath(String),
    /// Invalid device handle
    InvalidDeviceHandle(String),
    /// Device not found
    DeviceNotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// Initialization failed
    InitializationFailed(String),
    /// Injection failed
    InjectionFailed(String),
    /// Constraint violation
    ConstraintViolation(String),
}

impl core::fmt::Display for DomainError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidDevicePath(path) => write!(f, "Invalid device path: {}", path),
            Self::InvalidDeviceHandle(msg) => write!(f, "Invalid device handle: {}", msg),
            Self::DeviceNotFound(msg) => write!(f, "Device not found: {}", msg),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Self::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            Self::InjectionFailed(msg) => write!(f, "Injection failed: {}", msg),
            Self::ConstraintViolation(msg) => write!(f, "Constraint violation: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_error_display() {
        let err = DomainError::InvalidDevicePath("/dev/invalid".into());
        assert_eq!(err.to_string(), "Invalid device path: /dev/invalid");

        let err = DomainError::PermissionDenied("Insufficient privileges".into());
        assert_eq!(err.to_string(), "Permission denied: Insufficient privileges");

        let err = DomainError::ConstraintViolation("Test violation".into());
        assert_eq!(err.to_string(), "Constraint violation: Test violation");
    }
}
