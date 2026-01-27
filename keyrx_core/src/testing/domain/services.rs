//! Domain services for Testing domain
//!
//! Services contain business logic that doesn't naturally fit in entities or value objects.

use alloc::string::String;
use alloc::vec::Vec;

use crate::config::KeyCode;
use crate::runtime::KeyEvent;

use super::aggregates::{MockDeviceAggregate, TestScenarioAggregate};
use super::entities::TestCaseEntity;
use super::events::{DomainEvent, DomainEventBus};
use super::repositories::TestResult;
use super::value_objects::{SeedVO, TimestampVO};
use super::DomainError;

/// Deterministic simulation service
///
/// Executes test scenarios with deterministic timing and RNG.
pub struct DeterministicSimulationService {
    /// Event bus for publishing domain events
    event_bus: DomainEventBus,
    /// Current simulation timestamp
    current_time: TimestampVO,
}

impl DeterministicSimulationService {
    /// Creates a new deterministic simulation service
    pub fn new() -> Self {
        Self {
            event_bus: DomainEventBus::new(),
            current_time: TimestampVO::zero(),
        }
    }

    /// Executes a test scenario
    pub fn execute_scenario(
        &mut self,
        scenario: &TestScenarioAggregate,
        device: &mut MockDeviceAggregate,
    ) -> Result<Vec<TestResult>, DomainError> {
        // Validate scenario
        scenario.validate()?;

        // Check if scenario is enabled
        if !scenario.is_enabled() {
            return Err(DomainError::ConstraintViolation(
                "Cannot execute disabled scenario".into(),
            ));
        }

        // Publish scenario started event
        self.event_bus.publish(DomainEvent::TestScenarioStarted {
            scenario_name: scenario.name().into(),
            timestamp_us: self.current_time.as_microseconds(),
        });

        let mut results = Vec::new();

        // Execute each test case
        for test_case in scenario.test_cases() {
            if !test_case.is_enabled() {
                continue;
            }

            let result = self.execute_test_case(scenario.name(), test_case, device)?;
            results.push(result);
        }

        // Publish scenario completed event
        self.event_bus.publish(DomainEvent::TestScenarioCompleted {
            scenario_name: scenario.name().into(),
            passed: results.iter().all(|r| r.passed),
            timestamp_us: self.current_time.as_microseconds(),
        });

        Ok(results)
    }

    /// Executes a single test case
    fn execute_test_case(
        &mut self,
        scenario_name: &str,
        test_case: &TestCaseEntity,
        device: &mut MockDeviceAggregate,
    ) -> Result<TestResult, DomainError> {
        let start_time = self.current_time;

        // Publish test case started event
        self.event_bus.publish(DomainEvent::TestCaseStarted {
            scenario_name: scenario_name.into(),
            test_case_name: test_case.name().into(),
            timestamp_us: start_time.as_microseconds(),
        });

        // Reset device state
        device.reset();

        // Execute test case logic (simplified - real implementation would run mappings)
        // For now, just advance time
        self.advance_time(1000);
        device.advance_time(1000);

        // Evaluate assertions
        let mut all_passed = true;
        let mut error_message = None;

        for assertion in test_case.assertions() {
            // Simplified assertion evaluation
            // Real implementation would evaluate against actual device output
            if let Err(e) = assertion.validate() {
                all_passed = false;
                error_message = Some(alloc::format!("{}", e));
                break;
            }
        }

        let end_time = self.current_time;
        let duration_us = end_time
            .duration_since(start_time)
            .unwrap_or(0);

        // Check execution time constraint
        if let Some(expected_duration) = test_case.expected_duration_ms() {
            let duration_ms = duration_us / 1000;
            if duration_ms > expected_duration {
                return Err(DomainError::ExecutionTimeout {
                    timeout_ms: expected_duration,
                    elapsed_ms: duration_ms,
                });
            }
        }

        let result = if all_passed {
            self.event_bus.publish(DomainEvent::TestCasePassed {
                scenario_name: scenario_name.into(),
                test_case_name: test_case.name().into(),
                duration_us,
                timestamp_us: end_time.as_microseconds(),
            });

            TestResult::success(
                scenario_name.into(),
                test_case.name().into(),
                duration_us,
                end_time.as_microseconds(),
            )
        } else {
            self.event_bus.publish(DomainEvent::TestCaseFailed {
                scenario_name: scenario_name.into(),
                test_case_name: test_case.name().into(),
                error_message: error_message.clone().unwrap_or_default(),
                timestamp_us: end_time.as_microseconds(),
            });

            TestResult::failure(
                scenario_name.into(),
                test_case.name().into(),
                duration_us,
                error_message.unwrap_or_else(|| "Unknown error".into()),
                end_time.as_microseconds(),
            )
        };

        Ok(result)
    }

    /// Advances the simulation time
    pub fn advance_time(&mut self, delta_us: u64) {
        self.current_time = self.current_time.advance(delta_us);
    }

    /// Gets the current simulation time
    pub fn current_time(&self) -> TimestampVO {
        self.current_time
    }

    /// Resets the simulation to time zero
    pub fn reset(&mut self) {
        self.current_time = TimestampVO::zero();
        self.event_bus.clear();
    }

    /// Gets the event bus
    pub fn event_bus(&self) -> &DomainEventBus {
        &self.event_bus
    }

    /// Drains events from the bus
    pub fn drain_events(&mut self) -> Vec<DomainEvent> {
        self.event_bus.drain()
    }
}

impl Default for DeterministicSimulationService {
    fn default() -> Self {
        Self::new()
    }
}

/// Property-based test generator service
///
/// Generates test cases using property-based testing strategies.
pub struct PropertyTestGeneratorService {
    /// Random seed for generation
    seed: SeedVO,
    /// Generated test case counter
    generated_count: u64,
}

impl PropertyTestGeneratorService {
    /// Creates a new property test generator
    pub fn new(seed: SeedVO) -> Self {
        Self {
            seed,
            generated_count: 0,
        }
    }

    /// Generates test cases for a given property
    pub fn generate_test_cases(
        &mut self,
        property_name: &str,
        count: usize,
    ) -> Result<Vec<TestCaseEntity>, DomainError> {
        if count == 0 {
            return Err(DomainError::ConstraintViolation(
                "Test case count must be > 0".into(),
            ));
        }

        let mut test_cases = Vec::new();

        for i in 0..count {
            let _derived_seed = self.seed.derive(self.generated_count + i as u64);

            let test_case = TestCaseEntity::new(
                self.generated_count + i as u64,
                alloc::format!("{}-{}", property_name, i),
                alloc::format!("Generated test case {} for property {}", i, property_name),
            );

            test_cases.push(test_case);
        }

        self.generated_count += count as u64;

        Ok(test_cases)
    }

    /// Generates random key events for testing
    pub fn generate_key_events(
        &mut self,
        count: usize,
        timestamp_start: TimestampVO,
    ) -> Result<Vec<KeyEvent>, DomainError> {
        if count == 0 {
            return Err(DomainError::ConstraintViolation(
                "Event count must be > 0".into(),
            ));
        }

        let mut events = Vec::new();
        let mut current_time = timestamp_start;

        // Simple deterministic key event generation
        let keys = [KeyCode::A, KeyCode::B, KeyCode::C, KeyCode::D, KeyCode::E];

        for i in 0..count {
            let key = keys[i % keys.len()];
            let is_press = i % 2 == 0;

            let event = if is_press {
                KeyEvent::press(key)
            } else {
                KeyEvent::release(key)
            }
            .with_timestamp(current_time.as_microseconds());

            events.push(event);

            // Advance time deterministically
            current_time = current_time.advance(10_000); // 10ms between events
        }

        Ok(events)
    }

    /// Gets the current seed
    pub fn seed(&self) -> SeedVO {
        self.seed
    }

    /// Gets the number of generated test cases
    pub fn generated_count(&self) -> u64 {
        self.generated_count
    }

    /// Resets the generator
    pub fn reset(&mut self, new_seed: SeedVO) {
        self.seed = new_seed;
        self.generated_count = 0;
    }
}

/// Coverage analysis service
///
/// Tracks test coverage across the codebase.
pub struct CoverageAnalysisService {
    /// Coverage data by component
    coverage_data: Vec<CoverageEntry>,
    /// Event bus for coverage events
    event_bus: DomainEventBus,
}

/// Coverage entry for a component
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageEntry {
    /// Component name
    pub component: String,
    /// Coverage percentage (0-100)
    pub coverage_percent: u8,
    /// Number of covered lines
    pub covered_lines: usize,
    /// Total number of lines
    pub total_lines: usize,
}

impl CoverageAnalysisService {
    /// Creates a new coverage analysis service
    pub fn new() -> Self {
        Self {
            coverage_data: Vec::new(),
            event_bus: DomainEventBus::new(),
        }
    }

    /// Records coverage for a component
    pub fn record_coverage(
        &mut self,
        component: String,
        covered_lines: usize,
        total_lines: usize,
    ) -> Result<(), DomainError> {
        if total_lines == 0 {
            return Err(DomainError::ConstraintViolation(
                "Total lines must be > 0".into(),
            ));
        }

        if covered_lines > total_lines {
            return Err(DomainError::ConstraintViolation(
                "Covered lines cannot exceed total lines".into(),
            ));
        }

        let coverage_percent = ((covered_lines as f64 / total_lines as f64) * 100.0) as u8;

        // Update or add coverage entry
        if let Some(entry) = self.coverage_data.iter_mut().find(|e| e.component == component) {
            entry.coverage_percent = coverage_percent;
            entry.covered_lines = covered_lines;
            entry.total_lines = total_lines;
        } else {
            self.coverage_data.push(CoverageEntry {
                component: component.clone(),
                coverage_percent,
                covered_lines,
                total_lines,
            });
        }

        // Publish coverage updated event
        self.event_bus.publish(DomainEvent::CoverageUpdated {
            component,
            coverage_percent,
            timestamp_us: 0, // Would use real timestamp in production
        });

        Ok(())
    }

    /// Validates coverage against threshold
    pub fn validate_coverage_threshold(
        &self,
        component: &str,
        threshold_percent: u8,
    ) -> Result<(), DomainError> {
        if threshold_percent > 100 {
            return Err(DomainError::ConstraintViolation(
                "Threshold must be <= 100".into(),
            ));
        }

        let entry = self
            .coverage_data
            .iter()
            .find(|e| e.component == component)
            .ok_or_else(|| DomainError::ScenarioNotFound(component.into()))?;

        if entry.coverage_percent < threshold_percent {
            return Err(DomainError::CoverageThresholdNotMet {
                expected: threshold_percent,
                actual: entry.coverage_percent,
                component: component.into(),
            });
        }

        Ok(())
    }

    /// Gets coverage data for all components
    pub fn get_all_coverage(&self) -> &[CoverageEntry] {
        &self.coverage_data
    }

    /// Gets coverage for a specific component
    pub fn get_coverage(&self, component: &str) -> Option<&CoverageEntry> {
        self.coverage_data.iter().find(|e| e.component == component)
    }

    /// Calculates overall coverage percentage
    pub fn overall_coverage(&self) -> u8 {
        if self.coverage_data.is_empty() {
            return 0;
        }

        let total_covered: usize = self.coverage_data.iter().map(|e| e.covered_lines).sum();
        let total_lines: usize = self.coverage_data.iter().map(|e| e.total_lines).sum();

        if total_lines == 0 {
            0
        } else {
            ((total_covered as f64 / total_lines as f64) * 100.0) as u8
        }
    }

    /// Drains events from the bus
    pub fn drain_events(&mut self) -> Vec<DomainEvent> {
        self.event_bus.drain()
    }
}

impl Default for CoverageAnalysisService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_simulation_service() {
        let mut service = DeterministicSimulationService::new();

        assert_eq!(service.current_time(), TimestampVO::zero());

        service.advance_time(1000);
        assert_eq!(service.current_time().as_microseconds(), 1000);

        service.reset();
        assert_eq!(service.current_time(), TimestampVO::zero());
    }

    #[test]
    fn test_property_test_generator_service() {
        let seed = SeedVO::new(12345);
        let mut generator = PropertyTestGeneratorService::new(seed);

        let test_cases = generator.generate_test_cases("property-1", 5).unwrap();
        assert_eq!(test_cases.len(), 5);
        assert_eq!(generator.generated_count(), 5);

        // Empty count should fail
        assert!(generator.generate_test_cases("property-2", 0).is_err());
    }

    #[test]
    fn test_property_test_generator_key_events() {
        let seed = SeedVO::new(12345);
        let mut generator = PropertyTestGeneratorService::new(seed);

        let events = generator
            .generate_key_events(10, TimestampVO::new(1000))
            .unwrap();
        assert_eq!(events.len(), 10);

        // Events should have increasing timestamps
        for i in 1..events.len() {
            assert!(events[i].timestamp_us() > events[i - 1].timestamp_us());
        }
    }

    #[test]
    fn test_coverage_analysis_service() {
        let mut service = CoverageAnalysisService::new();

        // Record coverage
        service
            .record_coverage("keyrx_core".into(), 800, 1000)
            .unwrap();

        let entry = service.get_coverage("keyrx_core").unwrap();
        assert_eq!(entry.coverage_percent, 80);
        assert_eq!(entry.covered_lines, 800);
        assert_eq!(entry.total_lines, 1000);

        // Validate threshold
        assert!(service
            .validate_coverage_threshold("keyrx_core", 75)
            .is_ok());
        assert!(service
            .validate_coverage_threshold("keyrx_core", 85)
            .is_err());
    }

    #[test]
    fn test_coverage_analysis_overall() {
        let mut service = CoverageAnalysisService::new();

        service
            .record_coverage("component1".into(), 80, 100)
            .unwrap();
        service
            .record_coverage("component2".into(), 90, 100)
            .unwrap();

        let overall = service.overall_coverage();
        assert_eq!(overall, 85); // (80 + 90) / (100 + 100) = 85%
    }

    #[test]
    fn test_coverage_analysis_validation() {
        let mut service = CoverageAnalysisService::new();

        // Invalid inputs
        assert!(service.record_coverage("test".into(), 0, 0).is_err());
        assert!(service.record_coverage("test".into(), 101, 100).is_err());
        assert!(service
            .validate_coverage_threshold("test", 101)
            .is_err());
    }
}
