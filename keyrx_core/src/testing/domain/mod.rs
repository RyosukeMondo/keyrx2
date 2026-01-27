//! Domain-Driven Design module for Testing domain
//!
//! This module implements DDD patterns for the testing infrastructure:
//! - Aggregates: TestScenarioAggregate, MockDeviceAggregate
//! - Entities: TestCaseEntity, AssertionEntity
//! - Value Objects: TimestampVO, SeedVO
//! - Domain Services: DeterministicSimulationService, PropertyTestGeneratorService, CoverageAnalysisService
//! - Repositories: TestScenarioRepository, TestResultRepository

pub mod aggregates;
pub mod entities;
pub mod events;
pub mod repositories;
pub mod services;
pub mod value_objects;

// Re-export key types
pub use aggregates::{MockDeviceAggregate, TestScenarioAggregate};
pub use entities::{AssertionEntity, TestCaseEntity};
pub use events::DomainEvent;
pub use repositories::{TestResultRepository, TestScenarioRepository};
pub use services::{
    CoverageAnalysisService, DeterministicSimulationService, PropertyTestGeneratorService,
};
pub use value_objects::{SeedVO, TimestampVO};

/// Domain error type for Testing domain
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    /// Test scenario not found
    ScenarioNotFound(alloc::string::String),
    /// Invalid test case configuration
    InvalidTestCase(alloc::string::String),
    /// Assertion failed
    AssertionFailed {
        expected: alloc::string::String,
        actual: alloc::string::String,
    },
    /// Test execution timeout
    ExecutionTimeout {
        timeout_ms: u64,
        elapsed_ms: u64,
    },
    /// Coverage threshold not met
    CoverageThresholdNotMet {
        expected: u8,
        actual: u8,
        component: alloc::string::String,
    },
    /// Constraint violation
    ConstraintViolation(alloc::string::String),
}

impl core::fmt::Display for DomainError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ScenarioNotFound(name) => write!(f, "Test scenario not found: {}", name),
            Self::InvalidTestCase(msg) => write!(f, "Invalid test case: {}", msg),
            Self::AssertionFailed { expected, actual } => {
                write!(f, "Assertion failed: expected {}, got {}", expected, actual)
            }
            Self::ExecutionTimeout {
                timeout_ms,
                elapsed_ms,
            } => write!(
                f,
                "Test execution timeout: expected {}ms, took {}ms",
                timeout_ms, elapsed_ms
            ),
            Self::CoverageThresholdNotMet {
                expected,
                actual,
                component,
            } => write!(
                f,
                "Coverage threshold not met for {}: expected {}%, got {}%",
                component, expected, actual
            ),
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
        let err = DomainError::ScenarioNotFound("test-scenario".into());
        assert_eq!(err.to_string(), "Test scenario not found: test-scenario");

        let err = DomainError::AssertionFailed {
            expected: "A".into(),
            actual: "B".into(),
        };
        assert_eq!(err.to_string(), "Assertion failed: expected A, got B");

        let err = DomainError::ExecutionTimeout {
            timeout_ms: 1000,
            elapsed_ms: 1500,
        };
        assert_eq!(
            err.to_string(),
            "Test execution timeout: expected 1000ms, took 1500ms"
        );

        let err = DomainError::CoverageThresholdNotMet {
            expected: 80,
            actual: 75,
            component: "keyrx_core".into(),
        };
        assert_eq!(
            err.to_string(),
            "Coverage threshold not met for keyrx_core: expected 80%, got 75%"
        );
    }
}
