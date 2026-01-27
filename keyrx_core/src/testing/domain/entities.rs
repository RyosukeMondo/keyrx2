//! Domain entities for Testing domain
//!
//! Entities have unique identity and lifecycle.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::config::KeyCode;

use super::DomainError;

/// TestCase entity with unique identity
///
/// Represents an individual test case within a scenario.
#[derive(Clone)]
pub struct TestCaseEntity {
    /// Unique identifier for this test case
    id: u64,
    /// Test case name
    name: String,
    /// Description of what this test case validates
    description: String,
    /// Assertions to evaluate
    assertions: Vec<AssertionEntity>,
    /// Expected execution time in milliseconds
    expected_duration_ms: Option<u64>,
    /// Whether this test case is enabled
    enabled: bool,
    /// Timestamp when this entity was created (microseconds)
    created_at: u64,
}

impl TestCaseEntity {
    /// Creates a new TestCase entity
    pub fn new(id: u64, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
            assertions: Vec::new(),
            expected_duration_ms: None,
            enabled: true,
            created_at: 0,
        }
    }

    /// Creates a new TestCase entity with timestamp
    pub fn with_timestamp(id: u64, name: String, description: String, timestamp: u64) -> Self {
        Self {
            id,
            name,
            description,
            assertions: Vec::new(),
            expected_duration_ms: None,
            enabled: true,
            created_at: timestamp,
        }
    }

    /// Gets the entity ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Gets the test case name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Gets the assertions
    pub fn assertions(&self) -> &[AssertionEntity] {
        &self.assertions
    }

    /// Gets the expected duration
    pub fn expected_duration_ms(&self) -> Option<u64> {
        self.expected_duration_ms
    }

    /// Checks if this test case is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Gets the creation timestamp
    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    /// Sets the expected duration
    pub fn set_expected_duration(&mut self, duration_ms: u64) {
        self.expected_duration_ms = Some(duration_ms);
    }

    /// Enables this test case
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disables this test case
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Adds an assertion to this test case
    pub fn add_assertion(&mut self, assertion: AssertionEntity) -> Result<(), DomainError> {
        // Validate assertion
        assertion.validate()?;

        self.assertions.push(assertion);
        Ok(())
    }

    /// Validates this test case
    pub fn validate(&self) -> Result<(), DomainError> {
        // Name must not be empty
        if self.name.is_empty() {
            return Err(DomainError::InvalidTestCase(
                "Test case name cannot be empty".into(),
            ));
        }

        // Must have at least one assertion
        if self.assertions.is_empty() {
            return Err(DomainError::InvalidTestCase(format!(
                "Test case '{}' must have at least one assertion",
                self.name
            )));
        }

        // Validate all assertions
        for assertion in &self.assertions {
            assertion.validate()?;
        }

        Ok(())
    }
}

/// Assertion entity representing a test assertion
///
/// Assertions define expected outcomes for test cases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionEntity {
    /// Unique identifier
    id: u64,
    /// Type of assertion
    assertion_type: AssertionType,
    /// Description of what this assertion validates
    description: String,
}

/// Types of assertions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssertionType {
    /// Assert that an output key matches expected
    OutputKeyEquals {
        expected: KeyCode,
    },
    /// Assert that output count matches expected
    OutputCountEquals {
        expected: usize,
    },
    /// Assert that a modifier state matches expected
    ModifierStateEquals {
        bit: u8,
        expected: bool,
    },
    /// Assert that execution time is within bounds
    ExecutionTimeWithinBounds {
        max_ms: u64,
    },
    /// Assert that a specific event occurred
    EventOccurred {
        event_type: String,
    },
    /// Custom assertion with expected/actual values
    Custom {
        expected: String,
        actual: String,
    },
}

impl AssertionEntity {
    /// Creates a new assertion for output key equality
    pub fn output_key_equals(id: u64, expected: KeyCode, description: String) -> Self {
        Self {
            id,
            assertion_type: AssertionType::OutputKeyEquals { expected },
            description,
        }
    }

    /// Creates a new assertion for output count equality
    pub fn output_count_equals(id: u64, expected: usize, description: String) -> Self {
        Self {
            id,
            assertion_type: AssertionType::OutputCountEquals { expected },
            description,
        }
    }

    /// Creates a new assertion for modifier state equality
    pub fn modifier_state_equals(id: u64, bit: u8, expected: bool, description: String) -> Self {
        Self {
            id,
            assertion_type: AssertionType::ModifierStateEquals { bit, expected },
            description,
        }
    }

    /// Creates a new assertion for execution time bounds
    pub fn execution_time_within_bounds(id: u64, max_ms: u64, description: String) -> Self {
        Self {
            id,
            assertion_type: AssertionType::ExecutionTimeWithinBounds { max_ms },
            description,
        }
    }

    /// Gets the assertion ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Gets the assertion type
    pub fn assertion_type(&self) -> &AssertionType {
        &self.assertion_type
    }

    /// Gets the description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Validates this assertion
    pub fn validate(&self) -> Result<(), DomainError> {
        // Description must not be empty
        if self.description.is_empty() {
            return Err(DomainError::ConstraintViolation(
                "Assertion description cannot be empty".into(),
            ));
        }

        // Validate assertion-specific constraints
        match &self.assertion_type {
            AssertionType::ModifierStateEquals { bit, .. } => {
                if *bit >= 255 {
                    return Err(DomainError::ConstraintViolation(
                        "Modifier bit must be < 255".into(),
                    ));
                }
            }
            AssertionType::ExecutionTimeWithinBounds { max_ms } => {
                if *max_ms == 0 {
                    return Err(DomainError::ConstraintViolation(
                        "Execution time bound must be > 0".into(),
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Evaluates this assertion against actual values
    pub fn evaluate_output_key(&self, actual: KeyCode) -> Result<(), DomainError> {
        match &self.assertion_type {
            AssertionType::OutputKeyEquals { expected } => {
                if *expected != actual {
                    return Err(DomainError::AssertionFailed {
                        expected: alloc::format!("{:?}", expected),
                        actual: alloc::format!("{:?}", actual),
                    });
                }
                Ok(())
            }
            _ => Err(DomainError::ConstraintViolation(
                "Wrong assertion type for output key evaluation".into(),
            )),
        }
    }

    /// Evaluates this assertion against output count
    pub fn evaluate_output_count(&self, actual: usize) -> Result<(), DomainError> {
        match &self.assertion_type {
            AssertionType::OutputCountEquals { expected } => {
                if *expected != actual {
                    return Err(DomainError::AssertionFailed {
                        expected: alloc::format!("{}", expected),
                        actual: alloc::format!("{}", actual),
                    });
                }
                Ok(())
            }
            _ => Err(DomainError::ConstraintViolation(
                "Wrong assertion type for output count evaluation".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_case_entity_creation() {
        let test_case = TestCaseEntity::new(1, "test-1".into(), "Test description".into());

        assert_eq!(test_case.id(), 1);
        assert_eq!(test_case.name(), "test-1");
        assert_eq!(test_case.description(), "Test description");
        assert!(test_case.is_enabled());
        assert_eq!(test_case.assertions().len(), 0);
    }

    #[test]
    fn test_test_case_entity_add_assertion() {
        let mut test_case = TestCaseEntity::new(1, "test-1".into(), "Test description".into());

        let assertion =
            AssertionEntity::output_key_equals(1, KeyCode::A, "Output should be A".into());

        assert!(test_case.add_assertion(assertion).is_ok());
        assert_eq!(test_case.assertions().len(), 1);
    }

    #[test]
    fn test_test_case_entity_validation() {
        let test_case = TestCaseEntity::new(1, "test-1".into(), "Test description".into());

        // Should fail - no assertions
        assert!(test_case.validate().is_err());
    }

    #[test]
    fn test_test_case_entity_enable_disable() {
        let mut test_case = TestCaseEntity::new(1, "test-1".into(), "Test description".into());

        assert!(test_case.is_enabled());

        test_case.disable();
        assert!(!test_case.is_enabled());

        test_case.enable();
        assert!(test_case.is_enabled());
    }

    #[test]
    fn test_assertion_entity_output_key_equals() {
        let assertion =
            AssertionEntity::output_key_equals(1, KeyCode::A, "Output should be A".into());

        assert_eq!(assertion.id(), 1);
        assert_eq!(assertion.description(), "Output should be A");

        // Validation should pass
        assert!(assertion.validate().is_ok());

        // Evaluation should pass
        assert!(assertion.evaluate_output_key(KeyCode::A).is_ok());

        // Evaluation should fail
        assert!(assertion.evaluate_output_key(KeyCode::B).is_err());
    }

    #[test]
    fn test_assertion_entity_output_count_equals() {
        let assertion = AssertionEntity::output_count_equals(1, 5, "Should have 5 outputs".into());

        assert!(assertion.validate().is_ok());

        // Evaluation should pass
        assert!(assertion.evaluate_output_count(5).is_ok());

        // Evaluation should fail
        assert!(assertion.evaluate_output_count(3).is_err());
    }

    #[test]
    fn test_assertion_entity_modifier_state_validation() {
        let valid = AssertionEntity::modifier_state_equals(
            1,
            10,
            true,
            "Modifier 10 should be active".into(),
        );
        assert!(valid.validate().is_ok());

        let invalid = AssertionEntity::modifier_state_equals(
            2,
            255,
            true,
            "Invalid bit".into(),
        );
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_assertion_entity_execution_time_validation() {
        let valid =
            AssertionEntity::execution_time_within_bounds(1, 1000, "Should finish in 1s".into());
        assert!(valid.validate().is_ok());

        let invalid = AssertionEntity::execution_time_within_bounds(2, 0, "Invalid time".into());
        assert!(invalid.validate().is_err());
    }
}
