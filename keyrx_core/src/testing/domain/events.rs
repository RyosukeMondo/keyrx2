//! Domain events for Testing domain
//!
//! Domain events represent things that have happened in the domain.

use alloc::string::String;
use alloc::vec::Vec;

/// Domain event for Testing domain
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    /// A test scenario was started
    TestScenarioStarted {
        scenario_name: String,
        timestamp_us: u64,
    },
    /// A test scenario was completed
    TestScenarioCompleted {
        scenario_name: String,
        passed: bool,
        timestamp_us: u64,
    },
    /// A test case was started
    TestCaseStarted {
        scenario_name: String,
        test_case_name: String,
        timestamp_us: u64,
    },
    /// A test case passed
    TestCasePassed {
        scenario_name: String,
        test_case_name: String,
        duration_us: u64,
        timestamp_us: u64,
    },
    /// A test case failed
    TestCaseFailed {
        scenario_name: String,
        test_case_name: String,
        error_message: String,
        timestamp_us: u64,
    },
    /// Coverage was updated for a component
    CoverageUpdated {
        component: String,
        coverage_percent: u8,
        timestamp_us: u64,
    },
    /// A mock device was created
    MockDeviceCreated {
        device_id: String,
        timestamp_us: u64,
    },
    /// A mock device was reset
    MockDeviceReset {
        device_id: String,
        timestamp_us: u64,
    },
    /// An assertion was evaluated
    AssertionEvaluated {
        test_case_name: String,
        assertion_id: u64,
        passed: bool,
        timestamp_us: u64,
    },
}

impl DomainEvent {
    /// Gets the timestamp of this event
    pub fn timestamp(&self) -> u64 {
        match self {
            Self::TestScenarioStarted { timestamp_us, .. } => *timestamp_us,
            Self::TestScenarioCompleted { timestamp_us, .. } => *timestamp_us,
            Self::TestCaseStarted { timestamp_us, .. } => *timestamp_us,
            Self::TestCasePassed { timestamp_us, .. } => *timestamp_us,
            Self::TestCaseFailed { timestamp_us, .. } => *timestamp_us,
            Self::CoverageUpdated { timestamp_us, .. } => *timestamp_us,
            Self::MockDeviceCreated { timestamp_us, .. } => *timestamp_us,
            Self::MockDeviceReset { timestamp_us, .. } => *timestamp_us,
            Self::AssertionEvaluated { timestamp_us, .. } => *timestamp_us,
        }
    }

    /// Gets a human-readable name for this event type
    pub fn event_type_name(&self) -> &'static str {
        match self {
            Self::TestScenarioStarted { .. } => "TestScenarioStarted",
            Self::TestScenarioCompleted { .. } => "TestScenarioCompleted",
            Self::TestCaseStarted { .. } => "TestCaseStarted",
            Self::TestCasePassed { .. } => "TestCasePassed",
            Self::TestCaseFailed { .. } => "TestCaseFailed",
            Self::CoverageUpdated { .. } => "CoverageUpdated",
            Self::MockDeviceCreated { .. } => "MockDeviceCreated",
            Self::MockDeviceReset { .. } => "MockDeviceReset",
            Self::AssertionEvaluated { .. } => "AssertionEvaluated",
        }
    }

    /// Checks if this is a test success event
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            Self::TestCasePassed { .. }
                | Self::TestScenarioCompleted { passed: true, .. }
                | Self::AssertionEvaluated { passed: true, .. }
        )
    }

    /// Checks if this is a test failure event
    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            Self::TestCaseFailed { .. }
                | Self::TestScenarioCompleted { passed: false, .. }
                | Self::AssertionEvaluated { passed: false, .. }
        )
    }
}

/// Event bus for domain events
///
/// Collects and dispatches domain events.
pub struct DomainEventBus {
    events: Vec<DomainEvent>,
}

impl DomainEventBus {
    /// Creates a new event bus
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Publishes an event to the bus
    pub fn publish(&mut self, event: DomainEvent) {
        self.events.push(event);
    }

    /// Gets all events
    pub fn events(&self) -> &[DomainEvent] {
        &self.events
    }

    /// Drains all events from the bus
    pub fn drain(&mut self) -> Vec<DomainEvent> {
        core::mem::take(&mut self.events)
    }

    /// Clears all events
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Gets events by type
    pub fn get_by_type(&self, event_type: &str) -> Vec<&DomainEvent> {
        self.events
            .iter()
            .filter(|e| e.event_type_name() == event_type)
            .collect()
    }

    /// Gets all success events
    pub fn get_successes(&self) -> Vec<&DomainEvent> {
        self.events.iter().filter(|e| e.is_success()).collect()
    }

    /// Gets all failure events
    pub fn get_failures(&self) -> Vec<&DomainEvent> {
        self.events.iter().filter(|e| e.is_failure()).collect()
    }

    /// Counts events by type
    pub fn count_by_type(&self, event_type: &str) -> usize {
        self.events
            .iter()
            .filter(|e| e.event_type_name() == event_type)
            .count()
    }
}

impl Default for DomainEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_event_timestamp() {
        let event = DomainEvent::TestScenarioStarted {
            scenario_name: "test".into(),
            timestamp_us: 1000,
        };
        assert_eq!(event.timestamp(), 1000);

        let event = DomainEvent::TestCasePassed {
            scenario_name: "test".into(),
            test_case_name: "case1".into(),
            duration_us: 500,
            timestamp_us: 2000,
        };
        assert_eq!(event.timestamp(), 2000);
    }

    #[test]
    fn test_domain_event_type_name() {
        let event = DomainEvent::TestScenarioStarted {
            scenario_name: "test".into(),
            timestamp_us: 1000,
        };
        assert_eq!(event.event_type_name(), "TestScenarioStarted");

        let event = DomainEvent::TestCaseFailed {
            scenario_name: "test".into(),
            test_case_name: "case1".into(),
            error_message: "Failed".into(),
            timestamp_us: 2000,
        };
        assert_eq!(event.event_type_name(), "TestCaseFailed");
    }

    #[test]
    fn test_domain_event_success_failure() {
        let success = DomainEvent::TestCasePassed {
            scenario_name: "test".into(),
            test_case_name: "case1".into(),
            duration_us: 500,
            timestamp_us: 1000,
        };
        assert!(success.is_success());
        assert!(!success.is_failure());

        let failure = DomainEvent::TestCaseFailed {
            scenario_name: "test".into(),
            test_case_name: "case1".into(),
            error_message: "Failed".into(),
            timestamp_us: 2000,
        };
        assert!(!failure.is_success());
        assert!(failure.is_failure());

        let completed_success = DomainEvent::TestScenarioCompleted {
            scenario_name: "test".into(),
            passed: true,
            timestamp_us: 3000,
        };
        assert!(completed_success.is_success());

        let completed_failure = DomainEvent::TestScenarioCompleted {
            scenario_name: "test".into(),
            passed: false,
            timestamp_us: 4000,
        };
        assert!(completed_failure.is_failure());
    }

    #[test]
    fn test_event_bus() {
        let mut bus = DomainEventBus::new();

        bus.publish(DomainEvent::TestScenarioStarted {
            scenario_name: "test".into(),
            timestamp_us: 1000,
        });
        bus.publish(DomainEvent::TestCasePassed {
            scenario_name: "test".into(),
            test_case_name: "case1".into(),
            duration_us: 500,
            timestamp_us: 2000,
        });
        bus.publish(DomainEvent::TestCaseFailed {
            scenario_name: "test".into(),
            test_case_name: "case2".into(),
            error_message: "Failed".into(),
            timestamp_us: 3000,
        });

        assert_eq!(bus.events().len(), 3);

        let successes = bus.get_successes();
        assert_eq!(successes.len(), 1);

        let failures = bus.get_failures();
        assert_eq!(failures.len(), 1);

        let events = bus.drain();
        assert_eq!(events.len(), 3);
        assert_eq!(bus.events().len(), 0);
    }

    #[test]
    fn test_event_bus_count_by_type() {
        let mut bus = DomainEventBus::new();

        bus.publish(DomainEvent::TestScenarioStarted {
            scenario_name: "test1".into(),
            timestamp_us: 1000,
        });
        bus.publish(DomainEvent::TestScenarioStarted {
            scenario_name: "test2".into(),
            timestamp_us: 2000,
        });
        bus.publish(DomainEvent::TestCasePassed {
            scenario_name: "test1".into(),
            test_case_name: "case1".into(),
            duration_us: 500,
            timestamp_us: 3000,
        });

        assert_eq!(bus.count_by_type("TestScenarioStarted"), 2);
        assert_eq!(bus.count_by_type("TestCasePassed"), 1);
        assert_eq!(bus.count_by_type("TestCaseFailed"), 0);
    }

    #[test]
    fn test_event_bus_get_by_type() {
        let mut bus = DomainEventBus::new();

        bus.publish(DomainEvent::TestScenarioStarted {
            scenario_name: "test1".into(),
            timestamp_us: 1000,
        });
        bus.publish(DomainEvent::TestCasePassed {
            scenario_name: "test1".into(),
            test_case_name: "case1".into(),
            duration_us: 500,
            timestamp_us: 2000,
        });

        let started_events = bus.get_by_type("TestScenarioStarted");
        assert_eq!(started_events.len(), 1);

        let passed_events = bus.get_by_type("TestCasePassed");
        assert_eq!(passed_events.len(), 1);
    }
}
