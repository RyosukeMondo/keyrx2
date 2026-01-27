//! Domain aggregates for Testing domain
//!
//! Aggregates are clusters of domain objects that can be treated as a single unit.
//! They have a root entity and enforce consistency boundaries.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::runtime::KeyEvent;

use super::entities::TestCaseEntity;
use super::value_objects::{SeedVO, TimestampVO};
use super::DomainError;

/// TestScenario aggregate root
///
/// Encapsulates a complete test scenario with multiple test cases.
/// This is an aggregate because it maintains invariants across test cases,
/// assertions, and execution state.
#[derive(Clone)]
pub struct TestScenarioAggregate {
    /// Scenario identifier
    name: String,
    /// Description of what this scenario tests
    description: String,
    /// Test cases in this scenario
    test_cases: Vec<TestCaseEntity>,
    /// Random seed for deterministic execution
    seed: SeedVO,
    /// Whether this scenario is enabled
    enabled: bool,
    /// Tags for categorization
    tags: Vec<String>,
    /// Version counter for optimistic locking
    version: u64,
}

impl TestScenarioAggregate {
    /// Creates a new TestScenario aggregate
    pub fn new(name: String, description: String, seed: SeedVO) -> Self {
        Self {
            name,
            description,
            test_cases: Vec::new(),
            seed,
            enabled: true,
            tags: Vec::new(),
            version: 0,
        }
    }

    /// Gets the scenario name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the scenario description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Gets the test cases
    pub fn test_cases(&self) -> &[TestCaseEntity] {
        &self.test_cases
    }

    /// Gets the random seed
    pub fn seed(&self) -> SeedVO {
        self.seed
    }

    /// Checks if this scenario is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Gets the tags
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// Gets the version for optimistic locking
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Enables this scenario
    pub fn enable(&mut self) {
        self.enabled = true;
        self.version += 1;
    }

    /// Disables this scenario
    pub fn disable(&mut self) {
        self.enabled = false;
        self.version += 1;
    }

    /// Adds a test case to this scenario
    pub fn add_test_case(&mut self, test_case: TestCaseEntity) -> Result<(), DomainError> {
        // Validate test case
        if test_case.name().is_empty() {
            return Err(DomainError::InvalidTestCase(
                "Test case name cannot be empty".into(),
            ));
        }

        // Check for duplicate names
        if self
            .test_cases
            .iter()
            .any(|tc| tc.name() == test_case.name())
        {
            return Err(DomainError::InvalidTestCase(format!(
                "Test case '{}' already exists",
                test_case.name()
            )));
        }

        self.test_cases.push(test_case);
        self.version += 1;
        Ok(())
    }

    /// Removes a test case by name
    pub fn remove_test_case(&mut self, name: &str) -> Result<(), DomainError> {
        let index = self
            .test_cases
            .iter()
            .position(|tc| tc.name() == name)
            .ok_or_else(|| DomainError::InvalidTestCase(format!("Test case '{}' not found", name)))?;

        self.test_cases.remove(index);
        self.version += 1;
        Ok(())
    }

    /// Adds a tag to this scenario
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.version += 1;
        }
    }

    /// Validates this scenario
    pub fn validate(&self) -> Result<(), DomainError> {
        // Must have at least one test case
        if self.test_cases.is_empty() {
            return Err(DomainError::ConstraintViolation(
                "Scenario must have at least one test case".into(),
            ));
        }

        // Validate all test cases
        for test_case in &self.test_cases {
            test_case.validate()?;
        }

        Ok(())
    }
}

/// MockDevice aggregate root
///
/// Encapsulates a virtual test device that simulates keyboard input/output.
pub struct MockDeviceAggregate {
    /// Device identifier
    device_id: String,
    /// Input event queue
    input_queue: Vec<KeyEvent>,
    /// Output event history
    output_history: Vec<KeyEvent>,
    /// Current timestamp for deterministic time
    current_timestamp: TimestampVO,
    /// Whether this device is currently active
    active: bool,
    /// Version counter for optimistic locking
    version: u64,
}

impl MockDeviceAggregate {
    /// Creates a new MockDevice aggregate
    pub fn new(device_id: String, initial_timestamp: TimestampVO) -> Self {
        Self {
            device_id,
            input_queue: Vec::new(),
            output_history: Vec::new(),
            current_timestamp: initial_timestamp,
            active: true,
            version: 0,
        }
    }

    /// Gets the device ID
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    /// Gets the current timestamp
    pub fn current_timestamp(&self) -> TimestampVO {
        self.current_timestamp
    }

    /// Checks if this device is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Gets the version for optimistic locking
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Activates this device
    pub fn activate(&mut self) {
        self.active = true;
        self.version += 1;
    }

    /// Deactivates this device
    pub fn deactivate(&mut self) {
        self.active = false;
        self.version += 1;
    }

    /// Enqueues an input event
    pub fn enqueue_input(&mut self, event: KeyEvent) -> Result<(), DomainError> {
        if !self.active {
            return Err(DomainError::ConstraintViolation(
                "Cannot enqueue input on inactive device".into(),
            ));
        }

        self.input_queue.push(event);
        self.version += 1;
        Ok(())
    }

    /// Dequeues the next input event
    pub fn dequeue_input(&mut self) -> Option<KeyEvent> {
        if !self.active {
            return None;
        }

        if !self.input_queue.is_empty() {
            let event = self.input_queue.remove(0);
            self.version += 1;
            Some(event)
        } else {
            None
        }
    }

    /// Records an output event
    pub fn record_output(&mut self, event: KeyEvent) {
        self.output_history.push(event);
        self.version += 1;
    }

    /// Gets the input queue
    pub fn input_queue(&self) -> &[KeyEvent] {
        &self.input_queue
    }

    /// Gets the output history
    pub fn output_history(&self) -> &[KeyEvent] {
        &self.output_history
    }

    /// Advances the deterministic timestamp
    pub fn advance_time(&mut self, delta_us: u64) {
        self.current_timestamp = self.current_timestamp.advance(delta_us);
        self.version += 1;
    }

    /// Resets the device state
    pub fn reset(&mut self) {
        self.input_queue.clear();
        self.output_history.clear();
        self.version += 1;
    }

    /// Validates this device
    pub fn validate(&self) -> Result<(), DomainError> {
        // Device ID must not be empty
        if self.device_id.is_empty() {
            return Err(DomainError::ConstraintViolation(
                "Device ID cannot be empty".into(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::KeyCode;

    #[test]
    fn test_test_scenario_aggregate_creation() {
        let seed = SeedVO::new(12345);
        let scenario = TestScenarioAggregate::new(
            "test-scenario".into(),
            "Test description".into(),
            seed,
        );

        assert_eq!(scenario.name(), "test-scenario");
        assert_eq!(scenario.description(), "Test description");
        assert_eq!(scenario.seed(), seed);
        assert!(scenario.is_enabled());
        assert_eq!(scenario.test_cases().len(), 0);
    }

    #[test]
    fn test_test_scenario_aggregate_add_test_case() {
        let seed = SeedVO::new(12345);
        let mut scenario = TestScenarioAggregate::new(
            "test-scenario".into(),
            "Test description".into(),
            seed,
        );

        let test_case = TestCaseEntity::new(1, "test-case-1".into(), "Description".into());
        assert!(scenario.add_test_case(test_case).is_ok());
        assert_eq!(scenario.test_cases().len(), 1);

        // Duplicate name should fail
        let duplicate = TestCaseEntity::new(2, "test-case-1".into(), "Description".into());
        assert!(scenario.add_test_case(duplicate).is_err());
    }

    #[test]
    fn test_test_scenario_aggregate_validation() {
        let seed = SeedVO::new(12345);
        let scenario = TestScenarioAggregate::new(
            "test-scenario".into(),
            "Test description".into(),
            seed,
        );

        // Should fail - no test cases
        assert!(scenario.validate().is_err());
    }

    #[test]
    fn test_mock_device_aggregate_creation() {
        let timestamp = TimestampVO::new(1000);
        let device = MockDeviceAggregate::new("test-device".into(), timestamp);

        assert_eq!(device.device_id(), "test-device");
        assert_eq!(device.current_timestamp(), timestamp);
        assert!(device.is_active());
    }

    #[test]
    fn test_mock_device_aggregate_input_queue() {
        let timestamp = TimestampVO::new(1000);
        let mut device = MockDeviceAggregate::new("test-device".into(), timestamp);

        let event = KeyEvent::press(KeyCode::A).with_timestamp(1000);
        assert!(device.enqueue_input(event).is_ok());
        assert_eq!(device.input_queue().len(), 1);

        let dequeued = device.dequeue_input();
        assert!(dequeued.is_some());
        assert_eq!(device.input_queue().len(), 0);
    }

    #[test]
    fn test_mock_device_aggregate_output_history() {
        let timestamp = TimestampVO::new(1000);
        let mut device = MockDeviceAggregate::new("test-device".into(), timestamp);

        let event1 = KeyEvent::press(KeyCode::A).with_timestamp(1000);
        let event2 = KeyEvent::release(KeyCode::A).with_timestamp(2000);

        device.record_output(event1);
        device.record_output(event2);

        assert_eq!(device.output_history().len(), 2);
    }

    #[test]
    fn test_mock_device_aggregate_time_advancement() {
        let timestamp = TimestampVO::new(1000);
        let mut device = MockDeviceAggregate::new("test-device".into(), timestamp);

        device.advance_time(500);
        assert_eq!(device.current_timestamp().as_microseconds(), 1500);
    }

    #[test]
    fn test_mock_device_aggregate_inactive_state() {
        let timestamp = TimestampVO::new(1000);
        let mut device = MockDeviceAggregate::new("test-device".into(), timestamp);

        device.deactivate();
        assert!(!device.is_active());

        let event = KeyEvent::press(KeyCode::A).with_timestamp(1000);
        assert!(device.enqueue_input(event).is_err());
        assert!(device.dequeue_input().is_none());
    }
}
