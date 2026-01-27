//! Domain-Driven Design module for Configuration domain
//!
//! This module implements DDD patterns for configuration management:
//! - Aggregates: ProfileConfigAggregate, DeviceConfigAggregate, LayerAggregate
//! - Entities: ModifierEntity, LockEntity, MacroEntity
//! - Value Objects: LayerNameVO, ThresholdVO, ModifierIdVO
//! - Domain Services: ConfigMergingService, ConfigValidationService, ConfigMigrationService
//! - Repositories: ProfileConfigRepository, LayerRepository, MacroRepository

pub mod aggregates;
pub mod entities;
pub mod repositories;
pub mod services;
pub mod value_objects;

// Re-export key types
pub use aggregates::{DeviceConfigAggregate, LayerAggregate, ProfileConfigAggregate};
pub use entities::{LockEntity, MacroEntity, ModifierEntity};
pub use repositories::{LayerRepository, MacroRepository, ProfileConfigRepository};
pub use services::{ConfigMergingService, ConfigMigrationService, ConfigValidationService};
pub use value_objects::{LayerNameVO, ModifierIdVO, ThresholdVO};

/// Domain error type for Configuration domain
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigDomainError {
    /// Invalid layer name
    InvalidLayerName(alloc::string::String),
    /// Invalid threshold value
    InvalidThreshold(u16),
    /// Invalid modifier ID
    InvalidModifierId(u8),
    /// Layer not found
    LayerNotFound(alloc::string::String),
    /// Profile not found
    ProfileNotFound(alloc::string::String),
    /// Device not found
    DeviceNotFound(alloc::string::String),
    /// Macro not found
    MacroNotFound(alloc::string::String),
    /// Constraint violation
    ConstraintViolation(alloc::string::String),
    /// Migration error
    MigrationError(alloc::string::String),
}

impl core::fmt::Display for ConfigDomainError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidLayerName(name) => write!(f, "Invalid layer name: {}", name),
            Self::InvalidThreshold(val) => write!(f, "Invalid threshold: {}", val),
            Self::InvalidModifierId(id) => write!(f, "Invalid modifier ID: {}", id),
            Self::LayerNotFound(name) => write!(f, "Layer not found: {}", name),
            Self::ProfileNotFound(name) => write!(f, "Profile not found: {}", name),
            Self::DeviceNotFound(name) => write!(f, "Device not found: {}", name),
            Self::MacroNotFound(name) => write!(f, "Macro not found: {}", name),
            Self::ConstraintViolation(msg) => write!(f, "Constraint violation: {}", msg),
            Self::MigrationError(msg) => write!(f, "Migration error: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_config_domain_error_display() {
        let err = ConfigDomainError::InvalidLayerName("test".into());
        assert_eq!(err.to_string(), "Invalid layer name: test");

        let err = ConfigDomainError::InvalidThreshold(0);
        assert_eq!(err.to_string(), "Invalid threshold: 0");

        let err = ConfigDomainError::ConstraintViolation("Test violation".into());
        assert_eq!(err.to_string(), "Constraint violation: Test violation");
    }
}
