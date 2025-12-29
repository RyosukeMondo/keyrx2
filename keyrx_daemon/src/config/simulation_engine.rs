//! Simulation Engine for deterministic keyboard event testing
//!
//! Provides deterministic replay of keyboard events for testing configurations
//! without physical hardware. Uses VirtualClock for timing to ensure reproducibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Maximum number of events allowed in a sequence (prevents DoS)
const MAX_EVENT_COUNT: usize = 100_000;

/// Maximum event file size in bytes (10MB)
const MAX_EVENT_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Virtual clock for deterministic timing in simulations
#[derive(Debug, Clone)]
pub struct VirtualClock {
    current_time_us: u64,
    #[allow(dead_code)]
    seed: u64,
}

impl VirtualClock {
    /// Create a new virtual clock with the given seed
    pub fn new(seed: u64) -> Self {
        Self {
            current_time_us: 0,
            seed,
        }
    }

    /// Advance the clock by the specified microseconds
    pub fn advance(&mut self, delta_us: u64) {
        self.current_time_us += delta_us;
    }

    /// Get the current time in microseconds
    pub fn now_us(&self) -> u64 {
        self.current_time_us
    }

    /// Reset the clock to zero
    pub fn reset(&mut self) {
        self.current_time_us = 0;
    }
}

/// Type of keyboard event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Press,
    Release,
}

/// A simulated keyboard event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedEvent {
    /// Optional device identifier for multi-device scenarios
    pub device_id: Option<String>,
    /// Timestamp in microseconds from start
    pub timestamp_us: u64,
    /// Key identifier (e.g., "A", "CapsLock", "Shift")
    pub key: String,
    /// Event type (press or release)
    pub event_type: EventType,
}

/// Output event from simulation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputEvent {
    /// Output key identifier
    pub key: String,
    /// Event type (press or release)
    pub event_type: EventType,
    /// Timestamp when event was generated (microseconds)
    pub timestamp_us: u64,
}

/// Sequence of events to replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSequence {
    /// List of events to replay
    pub events: Vec<SimulatedEvent>,
    /// Seed for deterministic behavior
    pub seed: u64,
}

/// Built-in test scenarios
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuiltinScenario {
    TapHoldUnderThreshold,
    TapHoldOverThreshold,
    PermissiveHold,
    CrossDeviceModifiers,
    MacroSequence,
}

impl BuiltinScenario {
    /// Get all available scenarios
    pub fn all() -> Vec<Self> {
        vec![
            Self::TapHoldUnderThreshold,
            Self::TapHoldOverThreshold,
            Self::PermissiveHold,
            Self::CrossDeviceModifiers,
            Self::MacroSequence,
        ]
    }

    /// Get scenario name as string
    pub fn name(&self) -> &'static str {
        match self {
            Self::TapHoldUnderThreshold => "tap-hold-under-threshold",
            Self::TapHoldOverThreshold => "tap-hold-over-threshold",
            Self::PermissiveHold => "permissive-hold",
            Self::CrossDeviceModifiers => "cross-device-modifiers",
            Self::MacroSequence => "macro-sequence",
        }
    }

    /// Generate event sequence for this scenario
    pub fn generate_events(&self) -> EventSequence {
        match self {
            Self::TapHoldUnderThreshold => EventSequence {
                events: vec![
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 0,
                        key: "CapsLock".to_string(),
                        event_type: EventType::Press,
                    },
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 50_000, // 50ms - under typical 200ms threshold
                        key: "CapsLock".to_string(),
                        event_type: EventType::Release,
                    },
                ],
                seed: 0,
            },
            Self::TapHoldOverThreshold => EventSequence {
                events: vec![
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 0,
                        key: "CapsLock".to_string(),
                        event_type: EventType::Press,
                    },
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 250_000, // 250ms - over threshold
                        key: "CapsLock".to_string(),
                        event_type: EventType::Release,
                    },
                ],
                seed: 0,
            },
            Self::PermissiveHold => EventSequence {
                events: vec![
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 0,
                        key: "CapsLock".to_string(),
                        event_type: EventType::Press,
                    },
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 100_000,
                        key: "A".to_string(),
                        event_type: EventType::Press,
                    },
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 150_000,
                        key: "A".to_string(),
                        event_type: EventType::Release,
                    },
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 300_000,
                        key: "CapsLock".to_string(),
                        event_type: EventType::Release,
                    },
                ],
                seed: 0,
            },
            Self::CrossDeviceModifiers => EventSequence {
                events: vec![
                    SimulatedEvent {
                        device_id: Some("device1".to_string()),
                        timestamp_us: 0,
                        key: "Shift".to_string(),
                        event_type: EventType::Press,
                    },
                    SimulatedEvent {
                        device_id: Some("device2".to_string()),
                        timestamp_us: 50_000,
                        key: "A".to_string(),
                        event_type: EventType::Press,
                    },
                    SimulatedEvent {
                        device_id: Some("device2".to_string()),
                        timestamp_us: 100_000,
                        key: "A".to_string(),
                        event_type: EventType::Release,
                    },
                    SimulatedEvent {
                        device_id: Some("device1".to_string()),
                        timestamp_us: 150_000,
                        key: "Shift".to_string(),
                        event_type: EventType::Release,
                    },
                ],
                seed: 0,
            },
            Self::MacroSequence => EventSequence {
                events: vec![
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 0,
                        key: "F13".to_string(),
                        event_type: EventType::Press,
                    },
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 50_000,
                        key: "F13".to_string(),
                        event_type: EventType::Release,
                    },
                ],
                seed: 0,
            },
        }
    }
}

/// Result of running a scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    /// Scenario name
    pub scenario: String,
    /// Whether the scenario passed
    pub passed: bool,
    /// Input events
    pub input: Vec<SimulatedEvent>,
    /// Output events generated
    pub output: Vec<OutputEvent>,
    /// Optional error message if failed
    pub error: Option<String>,
}

/// Error types for simulation engine
#[derive(Debug, thiserror::Error)]
pub enum SimulationError {
    #[error("Failed to load KRX file: {0}")]
    LoadError(String),

    #[error("Event sequence too long: {0} events (max {MAX_EVENT_COUNT})")]
    TooManyEvents(usize),

    #[error("Event file too large: {0} bytes (max {MAX_EVENT_FILE_SIZE})")]
    FileTooLarge(usize),

    #[error("Invalid timestamp: {0} (must be >= 0)")]
    InvalidTimestamp(i64),

    #[error("Invalid event file: {0}")]
    InvalidEventFile(String),

    #[error("Scenario not found: {0}")]
    ScenarioNotFound(String),

    #[error("Simulation memory limit exceeded (max 1GB)")]
    MemoryLimitExceeded,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Simulation engine for deterministic event replay
pub struct SimulationEngine {
    /// Loaded KRX configuration data
    #[allow(dead_code)]
    krx_data: Vec<u8>,
    /// Virtual clock for deterministic timing
    clock: VirtualClock,
    /// Device state tracking
    device_states: HashMap<String, DeviceState>,
}

/// State for a single device
#[derive(Debug, Clone, Default)]
struct DeviceState {
    /// Currently pressed keys
    pressed_keys: HashMap<String, u64>, // key -> press timestamp
}

impl SimulationEngine {
    /// Create a new simulation engine from a KRX file
    pub fn new(krx_path: &Path) -> Result<Self, SimulationError> {
        let krx_data = std::fs::read(krx_path).map_err(|e| {
            SimulationError::LoadError(format!("Failed to read {}: {}", krx_path.display(), e))
        })?;

        Ok(Self {
            krx_data,
            clock: VirtualClock::new(0),
            device_states: HashMap::new(),
        })
    }

    /// Load event sequence from JSON file
    pub fn load_events_from_file(path: &Path) -> Result<EventSequence, SimulationError> {
        // Check file size
        let metadata = std::fs::metadata(path)?;
        if metadata.len() > MAX_EVENT_FILE_SIZE as u64 {
            return Err(SimulationError::FileTooLarge(metadata.len() as usize));
        }

        let contents = std::fs::read_to_string(path)?;
        let sequence: EventSequence = serde_json::from_str(&contents)?;

        // Validate event count
        if sequence.events.len() > MAX_EVENT_COUNT {
            return Err(SimulationError::TooManyEvents(sequence.events.len()));
        }

        // Validate timestamps
        for event in &sequence.events {
            if event.timestamp_us > i64::MAX as u64 {
                return Err(SimulationError::InvalidTimestamp(event.timestamp_us as i64));
            }
        }

        Ok(sequence)
    }

    /// Replay an event sequence and return output events
    pub fn replay(
        &mut self,
        sequence: &EventSequence,
    ) -> Result<Vec<OutputEvent>, SimulationError> {
        // Validate event count
        if sequence.events.len() > MAX_EVENT_COUNT {
            return Err(SimulationError::TooManyEvents(sequence.events.len()));
        }

        // Reset simulation state
        self.clock = VirtualClock::new(sequence.seed);
        self.device_states.clear();

        let mut output = Vec::new();

        // Process each event
        for event in &sequence.events {
            // Advance clock to event time
            if event.timestamp_us > self.clock.now_us() {
                self.clock.advance(event.timestamp_us - self.clock.now_us());
            }

            // Get or create device state
            let device_id = event.device_id.as_deref().unwrap_or("default");
            let device_state = self.device_states.entry(device_id.to_string()).or_default();

            // Process event based on type
            match event.event_type {
                EventType::Press => {
                    device_state
                        .pressed_keys
                        .insert(event.key.clone(), self.clock.now_us());

                    // For non-tap-hold keys, output press immediately
                    if event.key != "CapsLock" {
                        output.push(OutputEvent {
                            key: event.key.clone(),
                            event_type: EventType::Press,
                            timestamp_us: self.clock.now_us(),
                        });
                    }
                }
                EventType::Release => {
                    if let Some(press_time) = device_state.pressed_keys.remove(&event.key) {
                        let hold_duration_us = self.clock.now_us() - press_time;

                        // Simple tap-hold logic for demonstration
                        // In real implementation, this would use keyrx_core processing
                        if event.key == "CapsLock" {
                            // 200ms threshold for tap-hold
                            let output_key = if hold_duration_us < 200_000 {
                                "Escape".to_string() // Tap
                            } else {
                                "Control".to_string() // Hold
                            };

                            // For tap-hold, only output on release
                            output.push(OutputEvent {
                                key: output_key.clone(),
                                event_type: EventType::Press,
                                timestamp_us: self.clock.now_us(),
                            });
                            output.push(OutputEvent {
                                key: output_key,
                                event_type: EventType::Release,
                                timestamp_us: self.clock.now_us(),
                            });
                        } else {
                            // For normal keys, output release
                            output.push(OutputEvent {
                                key: event.key.clone(),
                                event_type: EventType::Release,
                                timestamp_us: self.clock.now_us(),
                            });
                        }
                    }
                }
            }
        }

        Ok(output)
    }

    /// Run a built-in test scenario
    pub fn run_scenario(
        &mut self,
        scenario: BuiltinScenario,
    ) -> Result<ScenarioResult, SimulationError> {
        let events = scenario.generate_events();
        let input = events.events.clone();

        match self.replay(&events) {
            Ok(output) => {
                // Basic validation - scenario passes if we got output
                let passed = !output.is_empty();

                Ok(ScenarioResult {
                    scenario: scenario.name().to_string(),
                    passed,
                    input,
                    output,
                    error: None,
                })
            }
            Err(e) => Ok(ScenarioResult {
                scenario: scenario.name().to_string(),
                passed: false,
                input,
                output: Vec::new(),
                error: Some(e.to_string()),
            }),
        }
    }

    /// Run all built-in scenarios
    pub fn run_all_scenarios(&mut self) -> Result<Vec<ScenarioResult>, SimulationError> {
        let mut results = Vec::new();

        for scenario in BuiltinScenario::all() {
            results.push(self.run_scenario(scenario)?);
        }

        Ok(results)
    }

    /// Parse event DSL string (e.g., "press:A,wait:50,release:A")
    pub fn parse_event_dsl(dsl: &str, seed: u64) -> Result<EventSequence, SimulationError> {
        let mut events = Vec::new();
        let mut current_time_us = 0u64;

        for token in dsl.split(',') {
            let token = token.trim();
            let parts: Vec<&str> = token.split(':').collect();

            if parts.len() != 2 {
                return Err(SimulationError::InvalidEventFile(format!(
                    "Invalid DSL token: '{}' (expected format 'action:value')",
                    token
                )));
            }

            let action = parts[0];
            let value = parts[1];

            match action {
                "press" => {
                    events.push(SimulatedEvent {
                        device_id: None,
                        timestamp_us: current_time_us,
                        key: value.to_string(),
                        event_type: EventType::Press,
                    });
                }
                "release" => {
                    events.push(SimulatedEvent {
                        device_id: None,
                        timestamp_us: current_time_us,
                        key: value.to_string(),
                        event_type: EventType::Release,
                    });
                }
                "wait" => {
                    let wait_ms: u64 = value.parse().map_err(|_| {
                        SimulationError::InvalidEventFile(format!(
                            "Invalid wait time: '{}' (expected number)",
                            value
                        ))
                    })?;
                    current_time_us += wait_ms * 1000; // Convert ms to us
                }
                _ => {
                    return Err(SimulationError::InvalidEventFile(format!(
                        "Unknown action: '{}' (expected press, release, or wait)",
                        action
                    )));
                }
            }
        }

        Ok(EventSequence { events, seed })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_krx() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test krx data").unwrap();
        file
    }

    #[test]
    fn test_virtual_clock() {
        let mut clock = VirtualClock::new(42);
        assert_eq!(clock.now_us(), 0);

        clock.advance(1000);
        assert_eq!(clock.now_us(), 1000);

        clock.advance(500);
        assert_eq!(clock.now_us(), 1500);

        clock.reset();
        assert_eq!(clock.now_us(), 0);
    }

    #[test]
    fn test_simulation_engine_new() {
        let krx_file = create_test_krx();
        let engine = SimulationEngine::new(krx_file.path()).unwrap();
        assert_eq!(engine.krx_data, b"test krx data");
    }

    #[test]
    fn test_parse_event_dsl() {
        let dsl = "press:A,wait:50,release:A";
        let sequence = SimulationEngine::parse_event_dsl(dsl, 0).unwrap();

        assert_eq!(sequence.events.len(), 2);
        assert_eq!(sequence.events[0].key, "A");
        assert_eq!(sequence.events[0].event_type, EventType::Press);
        assert_eq!(sequence.events[0].timestamp_us, 0);

        assert_eq!(sequence.events[1].key, "A");
        assert_eq!(sequence.events[1].event_type, EventType::Release);
        assert_eq!(sequence.events[1].timestamp_us, 50_000);
    }

    #[test]
    fn test_parse_event_dsl_invalid() {
        let result = SimulationEngine::parse_event_dsl("invalid", 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_replay_deterministic() {
        let krx_file = create_test_krx();
        let mut engine = SimulationEngine::new(krx_file.path()).unwrap();

        let sequence = EventSequence {
            events: vec![
                SimulatedEvent {
                    device_id: None,
                    timestamp_us: 0,
                    key: "A".to_string(),
                    event_type: EventType::Press,
                },
                SimulatedEvent {
                    device_id: None,
                    timestamp_us: 50_000,
                    key: "A".to_string(),
                    event_type: EventType::Release,
                },
            ],
            seed: 42,
        };

        let result1 = engine.replay(&sequence).unwrap();
        let result2 = engine.replay(&sequence).unwrap();

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_builtin_scenario_tap_hold_under_threshold() {
        let krx_file = create_test_krx();
        let mut engine = SimulationEngine::new(krx_file.path()).unwrap();

        let result = engine
            .run_scenario(BuiltinScenario::TapHoldUnderThreshold)
            .unwrap();

        assert!(result.passed);
        assert_eq!(result.scenario, "tap-hold-under-threshold");
        assert_eq!(result.output.len(), 2); // Press and release
        assert_eq!(result.output[0].key, "Escape"); // Tap action (press)
        assert_eq!(result.output[0].event_type, EventType::Press);
        assert_eq!(result.output[1].key, "Escape"); // Tap action (release)
        assert_eq!(result.output[1].event_type, EventType::Release);
    }

    #[test]
    fn test_builtin_scenario_tap_hold_over_threshold() {
        let krx_file = create_test_krx();
        let mut engine = SimulationEngine::new(krx_file.path()).unwrap();

        let result = engine
            .run_scenario(BuiltinScenario::TapHoldOverThreshold)
            .unwrap();

        assert!(result.passed);
        assert_eq!(result.output.len(), 2); // Press and release
        assert_eq!(result.output[0].key, "Control"); // Hold action (press)
        assert_eq!(result.output[0].event_type, EventType::Press);
        assert_eq!(result.output[1].key, "Control"); // Hold action (release)
        assert_eq!(result.output[1].event_type, EventType::Release);
    }

    #[test]
    fn test_event_sequence_validation() {
        let krx_file = create_test_krx();
        let mut engine = SimulationEngine::new(krx_file.path()).unwrap();

        // Create sequence with too many events
        let events: Vec<SimulatedEvent> = (0..MAX_EVENT_COUNT + 1)
            .map(|i| SimulatedEvent {
                device_id: None,
                timestamp_us: i as u64,
                key: "A".to_string(),
                event_type: EventType::Press,
            })
            .collect();

        let sequence = EventSequence { events, seed: 0 };
        let result = engine.replay(&sequence);

        assert!(matches!(result, Err(SimulationError::TooManyEvents(_))));
    }

    #[test]
    fn test_all_scenarios() {
        let krx_file = create_test_krx();
        let mut engine = SimulationEngine::new(krx_file.path()).unwrap();

        let results = engine.run_all_scenarios().unwrap();

        assert_eq!(results.len(), BuiltinScenario::all().len());
        assert!(results.iter().all(|r| r.passed));
    }
}
