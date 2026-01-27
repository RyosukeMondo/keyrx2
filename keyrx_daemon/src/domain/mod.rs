//! Domain-Driven Design module for Daemon domain
//!
//! This module implements DDD patterns for the daemon operations:
//! - Aggregates: DeviceAggregate, ProfileAggregate, SessionAggregate
//! - Entities: InputDeviceEntity, OutputDeviceEntity, WebSocketConnectionEntity
//! - Value Objects: DeviceSerialVO, ProfileNameVO, PortVO
//! - Domain Services: DeviceIdentificationService, ProfileSwitchingService, WebSocketBroadcastService
//! - Repositories: DeviceRepository, ProfileRepository, SettingsRepository (traits)

pub mod aggregates;
pub mod entities;
pub mod events;
pub mod repositories;
pub mod services;
pub mod value_objects;

// Re-export key types
pub use aggregates::{DeviceAggregate, ProfileAggregate, SessionAggregate};
pub use entities::{InputDeviceEntity, OutputDeviceEntity, WebSocketConnectionEntity};
pub use events::DomainEvent;
pub use repositories::{DeviceRepository, ProfileRepository, SettingsRepository};
pub use services::{
    DeviceIdentificationService, ProfileSwitchingService, WebSocketBroadcastService,
};
pub use value_objects::{DeviceSerialVO, PortVO, ProfileNameVO};

/// Domain error type for Daemon domain
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    /// Invalid device serial number
    InvalidDeviceSerial(String),
    /// Invalid profile name
    InvalidProfileName(String),
    /// Invalid port number
    InvalidPort(u16),
    /// Device not found
    DeviceNotFound(String),
    /// Profile not found
    ProfileNotFound(String),
    /// Device already connected
    DeviceAlreadyConnected(String),
    /// Profile already active
    ProfileAlreadyActive(String),
    /// Invalid state transition
    InvalidStateTransition { from: String, to: String },
    /// Constraint violation
    ConstraintViolation(String),
    /// WebSocket connection error
    WebSocketError(String),
}

impl core::fmt::Display for DomainError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidDeviceSerial(serial) => write!(f, "Invalid device serial: {}", serial),
            Self::InvalidProfileName(name) => write!(f, "Invalid profile name: {}", name),
            Self::InvalidPort(port) => write!(f, "Invalid port: {}", port),
            Self::DeviceNotFound(id) => write!(f, "Device not found: {}", id),
            Self::ProfileNotFound(name) => write!(f, "Profile not found: {}", name),
            Self::DeviceAlreadyConnected(id) => write!(f, "Device already connected: {}", id),
            Self::ProfileAlreadyActive(name) => write!(f, "Profile already active: {}", name),
            Self::InvalidStateTransition { from, to } => {
                write!(f, "Invalid state transition: {} -> {}", from, to)
            }
            Self::ConstraintViolation(msg) => write!(f, "Constraint violation: {}", msg),
            Self::WebSocketError(msg) => write!(f, "WebSocket error: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_error_display() {
        let err = DomainError::InvalidDeviceSerial("ABC123".into());
        assert_eq!(err.to_string(), "Invalid device serial: ABC123");

        let err = DomainError::InvalidPort(65535);
        assert_eq!(err.to_string(), "Invalid port: 65535");

        let err = DomainError::ConstraintViolation("Test violation".into());
        assert_eq!(err.to_string(), "Constraint violation: Test violation");
    }

    #[test]
    fn test_domain_error_equality() {
        let err1 = DomainError::InvalidPort(8080);
        let err2 = DomainError::InvalidPort(8080);
        let err3 = DomainError::InvalidPort(3000);

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }
}
