//! Testing infrastructure module
//!
//! This module provides domain-driven testing infrastructure including:
//! - Test scenario management
//! - Deterministic simulation
//! - Property-based test generation
//! - Coverage analysis

pub mod domain;

// Re-export key types for convenience
pub use domain::{
    AssertionEntity, CoverageAnalysisService, DeterministicSimulationService, DomainError,
    DomainEvent, MockDeviceAggregate, PropertyTestGeneratorService, SeedVO,
    TestCaseEntity, TestResultRepository, TestScenarioAggregate, TestScenarioRepository,
    TimestampVO,
};
