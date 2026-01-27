//! Domain-Driven Design module for Core domain
//!
//! This module implements DDD patterns for the platform-agnostic remapping logic:
//! - Aggregates: KeyMappingAggregate, StateAggregate
//! - Entities: KeyEvent, Action
//! - Value Objects: KeyCode, Condition
//! - Domain Services: EventProcessor, StateMachine
//! - Repositories: ConfigRepository (trait)

pub mod aggregates;
pub mod entities;
pub mod events;
pub mod repositories;
pub mod services;
pub mod value_objects;

// Re-export key types
pub use aggregates::{KeyMappingAggregate, StateAggregate};
pub use entities::{Action, KeyEventEntity};
pub use events::DomainEvent;
pub use repositories::{ConfigRepository, StateRepository};
pub use services::{EventProcessorService, StateMachineService};
pub use value_objects::{ConditionVO, KeyCodeVO};

/// Domain error type for Core domain
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    /// Invalid key code
    InvalidKeyCode(u16),
    /// Invalid state transition
    InvalidStateTransition {
        from: alloc::string::String,
        to: alloc::string::String,
    },
    /// Configuration not loaded
    ConfigurationNotLoaded,
    /// Constraint violation
    ConstraintViolation(alloc::string::String),
}

impl core::fmt::Display for DomainError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidKeyCode(code) => write!(f, "Invalid key code: {}", code),
            Self::InvalidStateTransition { from, to } => {
                write!(f, "Invalid state transition: {} -> {}", from, to)
            }
            Self::ConfigurationNotLoaded => write!(f, "Configuration not loaded"),
            Self::ConstraintViolation(msg) => write!(f, "Constraint violation: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_domain_error_display() {
        let err = DomainError::InvalidKeyCode(999);
        assert_eq!(err.to_string(), "Invalid key code: 999");

        let err = DomainError::ConstraintViolation("Test violation".into());
        assert_eq!(err.to_string(), "Constraint violation: Test violation");
    }
}
