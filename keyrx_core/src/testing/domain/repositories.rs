//! Domain repository traits for Testing domain
//!
//! Repositories provide an abstraction for data access.
//! These are traits that must be implemented by infrastructure layer.

use alloc::string::String;
use alloc::vec::Vec;

use super::aggregates::TestScenarioAggregate;
use super::DomainError;

/// Repository for test scenario data
///
/// Provides access to test scenarios and their configurations.
pub trait TestScenarioRepository {
    /// Loads a test scenario by name
    fn load(&self, name: &str) -> Result<TestScenarioAggregate, DomainError>;

    /// Lists all available test scenarios
    fn list(&self) -> Result<Vec<String>, DomainError>;

    /// Saves a test scenario
    fn save(&mut self, scenario: &TestScenarioAggregate) -> Result<(), DomainError>;

    /// Checks if a scenario exists
    fn exists(&self, name: &str) -> bool;

    /// Deletes a test scenario
    fn delete(&mut self, name: &str) -> Result<(), DomainError>;

    /// Gets all scenarios with a specific tag
    fn find_by_tag(&self, tag: &str) -> Result<Vec<TestScenarioAggregate>, DomainError>;
}

/// Test execution result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestResult {
    /// Scenario name
    pub scenario_name: String,
    /// Test case name
    pub test_case_name: String,
    /// Whether the test passed
    pub passed: bool,
    /// Duration in microseconds
    pub duration_us: u64,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Timestamp when executed
    pub executed_at: u64,
}

impl TestResult {
    /// Creates a new successful test result
    pub fn success(
        scenario_name: String,
        test_case_name: String,
        duration_us: u64,
        executed_at: u64,
    ) -> Self {
        Self {
            scenario_name,
            test_case_name,
            passed: true,
            duration_us,
            error_message: None,
            executed_at,
        }
    }

    /// Creates a new failed test result
    pub fn failure(
        scenario_name: String,
        test_case_name: String,
        duration_us: u64,
        error_message: String,
        executed_at: u64,
    ) -> Self {
        Self {
            scenario_name,
            test_case_name,
            passed: false,
            duration_us,
            error_message: Some(error_message),
            executed_at,
        }
    }
}

/// Repository for test result persistence
///
/// Provides access to test execution results and history.
pub trait TestResultRepository {
    /// Saves a test result
    fn save(&mut self, result: &TestResult) -> Result<(), DomainError>;

    /// Gets results for a specific scenario
    fn get_by_scenario(&self, scenario_name: &str) -> Result<Vec<TestResult>, DomainError>;

    /// Gets results for a specific test case
    fn get_by_test_case(
        &self,
        scenario_name: &str,
        test_case_name: &str,
    ) -> Result<Vec<TestResult>, DomainError>;

    /// Gets the most recent result for a test case
    fn get_latest(
        &self,
        scenario_name: &str,
        test_case_name: &str,
    ) -> Result<TestResult, DomainError>;

    /// Gets all failed results
    fn get_failures(&self) -> Result<Vec<TestResult>, DomainError>;

    /// Clears all results
    fn clear(&mut self) -> Result<(), DomainError>;

    /// Gets test statistics
    fn get_statistics(&self) -> Result<TestStatistics, DomainError>;
}

/// Test statistics
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestStatistics {
    /// Total number of test executions
    pub total_executions: usize,
    /// Number of passed tests
    pub passed_count: usize,
    /// Number of failed tests
    pub failed_count: usize,
    /// Average execution time in microseconds
    pub average_duration_us: u64,
    /// Total execution time in microseconds
    pub total_duration_us: u64,
}

impl TestStatistics {
    /// Creates new empty statistics
    pub fn new() -> Self {
        Self {
            total_executions: 0,
            passed_count: 0,
            failed_count: 0,
            average_duration_us: 0,
            total_duration_us: 0,
        }
    }

    /// Calculates the pass rate as a percentage
    pub fn pass_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            (self.passed_count as f64 / self.total_executions as f64) * 100.0
        }
    }

    /// Calculates the fail rate as a percentage
    pub fn fail_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            (self.failed_count as f64 / self.total_executions as f64) * 100.0
        }
    }
}

impl Default for TestStatistics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock implementation for testing TestScenarioRepository
    struct MockTestScenarioRepository {
        scenarios: Vec<TestScenarioAggregate>,
    }

    impl MockTestScenarioRepository {
        fn new() -> Self {
            Self {
                scenarios: Vec::new(),
            }
        }
    }

    impl TestScenarioRepository for MockTestScenarioRepository {
        fn load(&self, name: &str) -> Result<TestScenarioAggregate, DomainError> {
            self.scenarios
                .iter()
                .find(|s| s.name() == name)
                .cloned()
                .ok_or_else(|| DomainError::ScenarioNotFound(name.into()))
        }

        fn list(&self) -> Result<Vec<String>, DomainError> {
            Ok(self.scenarios.iter().map(|s| s.name().into()).collect())
        }

        fn save(&mut self, scenario: &TestScenarioAggregate) -> Result<(), DomainError> {
            // Remove existing scenario with same name
            self.scenarios.retain(|s| s.name() != scenario.name());
            // Add new scenario (clone for testing)
            // In real implementation, would persist to storage
            Ok(())
        }

        fn exists(&self, name: &str) -> bool {
            self.scenarios.iter().any(|s| s.name() == name)
        }

        fn delete(&mut self, name: &str) -> Result<(), DomainError> {
            let initial_len = self.scenarios.len();
            self.scenarios.retain(|s| s.name() != name);

            if self.scenarios.len() == initial_len {
                Err(DomainError::ScenarioNotFound(name.into()))
            } else {
                Ok(())
            }
        }

        fn find_by_tag(&self, tag: &str) -> Result<Vec<TestScenarioAggregate>, DomainError> {
            Ok(self
                .scenarios
                .iter()
                .filter(|s| s.tags().contains(&tag.into()))
                .cloned()
                .collect())
        }
    }

    /// Mock implementation for testing TestResultRepository
    struct MockTestResultRepository {
        results: Vec<TestResult>,
    }

    impl MockTestResultRepository {
        fn new() -> Self {
            Self {
                results: Vec::new(),
            }
        }
    }

    impl TestResultRepository for MockTestResultRepository {
        fn save(&mut self, result: &TestResult) -> Result<(), DomainError> {
            self.results.push(result.clone());
            Ok(())
        }

        fn get_by_scenario(&self, scenario_name: &str) -> Result<Vec<TestResult>, DomainError> {
            Ok(self
                .results
                .iter()
                .filter(|r| r.scenario_name == scenario_name)
                .cloned()
                .collect())
        }

        fn get_by_test_case(
            &self,
            scenario_name: &str,
            test_case_name: &str,
        ) -> Result<Vec<TestResult>, DomainError> {
            Ok(self
                .results
                .iter()
                .filter(|r| r.scenario_name == scenario_name && r.test_case_name == test_case_name)
                .cloned()
                .collect())
        }

        fn get_latest(
            &self,
            scenario_name: &str,
            test_case_name: &str,
        ) -> Result<TestResult, DomainError> {
            self.results
                .iter()
                .filter(|r| r.scenario_name == scenario_name && r.test_case_name == test_case_name)
                .max_by_key(|r| r.executed_at)
                .cloned()
                .ok_or_else(|| DomainError::ScenarioNotFound(scenario_name.into()))
        }

        fn get_failures(&self) -> Result<Vec<TestResult>, DomainError> {
            Ok(self
                .results
                .iter()
                .filter(|r| !r.passed)
                .cloned()
                .collect())
        }

        fn clear(&mut self) -> Result<(), DomainError> {
            self.results.clear();
            Ok(())
        }

        fn get_statistics(&self) -> Result<TestStatistics, DomainError> {
            let total = self.results.len();
            let passed = self.results.iter().filter(|r| r.passed).count();
            let failed = total - passed;
            let total_duration: u64 = self.results.iter().map(|r| r.duration_us).sum();
            let avg_duration = if total > 0 {
                total_duration / total as u64
            } else {
                0
            };

            Ok(TestStatistics {
                total_executions: total,
                passed_count: passed,
                failed_count: failed,
                average_duration_us: avg_duration,
                total_duration_us: total_duration,
            })
        }
    }

    #[test]
    fn test_test_result_creation() {
        let success = TestResult::success("scenario".into(), "test-case".into(), 1000, 5000);
        assert!(success.passed);
        assert_eq!(success.duration_us, 1000);
        assert!(success.error_message.is_none());

        let failure = TestResult::failure(
            "scenario".into(),
            "test-case".into(),
            1000,
            "Test failed".into(),
            5000,
        );
        assert!(!failure.passed);
        assert!(failure.error_message.is_some());
    }

    #[test]
    fn test_test_statistics() {
        let mut stats = TestStatistics::new();
        assert_eq!(stats.total_executions, 0);
        assert_eq!(stats.pass_rate(), 0.0);

        stats.total_executions = 10;
        stats.passed_count = 8;
        stats.failed_count = 2;

        assert_eq!(stats.pass_rate(), 80.0);
        assert_eq!(stats.fail_rate(), 20.0);
    }

    #[test]
    fn test_mock_result_repository() {
        let mut repo = MockTestResultRepository::new();

        let result1 = TestResult::success("scenario1".into(), "test1".into(), 1000, 1000);
        let result2 = TestResult::failure(
            "scenario1".into(),
            "test2".into(),
            2000,
            "Failed".into(),
            2000,
        );

        repo.save(&result1).unwrap();
        repo.save(&result2).unwrap();

        let by_scenario = repo.get_by_scenario("scenario1").unwrap();
        assert_eq!(by_scenario.len(), 2);

        let failures = repo.get_failures().unwrap();
        assert_eq!(failures.len(), 1);
        assert!(!failures[0].passed);

        let stats = repo.get_statistics().unwrap();
        assert_eq!(stats.total_executions, 2);
        assert_eq!(stats.passed_count, 1);
        assert_eq!(stats.failed_count, 1);
    }
}
