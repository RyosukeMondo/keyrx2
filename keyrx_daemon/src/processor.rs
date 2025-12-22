//! Event processor orchestrator.
//!
//! This module provides the `EventProcessor` struct, which orchestrates the
//! complete event processing pipeline: reading from input devices, processing
//! events through the runtime, and injecting output events.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐
//! │InputDevice  │ (e.g., MockInput, LinuxPlatform)
//! └──────┬──────┘
//!        │ KeyEvent
//!        ▼
//! ┌─────────────────────┐
//! │ EventProcessor      │
//! │  - KeyLookup        │ (find mapping for key)
//! │  - DeviceState      │ (modifier/lock state)
//! │  - process_event()  │ (apply mapping)
//! └──────┬──────────────┘
//!        │ Vec<KeyEvent>
//!        ▼
//! ┌─────────────┐
//! │OutputDevice │ (e.g., MockOutput, LinuxPlatform)
//! └─────────────┘
//! ```
//!
//! # Structured Logging
//!
//! The event processor emits structured JSON logs for observability:
//!
//! - `config_loaded` (INFO): Logged when processor is initialized
//!   - Fields: timestamp, level, service, event_type, context.mapping_count
//! - `key_processed` (DEBUG): Logged for each event processed
//!   - Fields: timestamp, level, service, event_type, context.input_key, context.output_keys, context.latency_us
//! - `state_transition` (DEBUG): Logged when modifier/lock state changes
//!   - Fields: timestamp, level, service, event_type, context.transition_type, context.modifier_id, context.lock_id
//! - `platform_error` (ERROR): Logged when device errors occur
//!   - Fields: timestamp, level, service, event_type, context.error
//!
//! All logs follow the schema: `{"timestamp":"...", "level":"...", "service":"keyrx_daemon", "event_type":"...", "context":{...}}`
//!
//! # Example
//!
//! ```no_run
//! use keyrx_daemon::processor::EventProcessor;
//! use keyrx_daemon::platform::{MockInput, MockOutput};
//! use keyrx_core::config::mappings::{DeviceConfig, DeviceIdentifier};
//! use keyrx_core::runtime::KeyEvent;
//! use keyrx_core::config::KeyCode;
//!
//! // Create mock devices for testing
//! let input = MockInput::new(vec![
//!     KeyEvent::Press(KeyCode::A),
//!     KeyEvent::Release(KeyCode::A),
//! ]);
//! let output = MockOutput::new();
//!
//! // Create device config (simplified example)
//! let config = DeviceConfig {
//!     identifier: DeviceIdentifier { pattern: String::from("*") },
//!     mappings: vec![/* ... */],
//! };
//!
//! // Create and run processor
//! let mut processor = EventProcessor::new(&config, input, output);
//! processor.run().unwrap();
//! ```

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use keyrx_core::config::DeviceConfig;
use keyrx_core::runtime::event::process_event;
use keyrx_core::runtime::{DeviceState, KeyLookup};
use log::{debug, error, info};
use thiserror::Error;

use crate::platform::{DeviceError, InputDevice, OutputDevice};

/// Errors that can occur during event processing.
#[derive(Debug, Error)]
pub enum ProcessorError {
    /// Error reading from input device.
    #[error("input device error: {0}")]
    Input(#[from] DeviceError),

    /// Error writing to output device.
    #[error("output device error: {0}")]
    Output(DeviceError),
}

/// Returns the current Unix timestamp in ISO 8601 format.
fn current_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| {
            let secs = d.as_secs();
            let nanos = d.subsec_nanos();
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
                1970 + secs / 31_557_600,
                ((secs % 31_557_600) / 2_629_800) + 1,
                ((secs % 2_629_800) / 86400) + 1,
                (secs % 86400) / 3600,
                (secs % 3600) / 60,
                secs % 60,
                nanos / 1_000_000
            )
        })
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00.000Z"))
}

/// Event processor orchestrator.
///
/// Coordinates the complete event processing pipeline:
/// 1. Read events from input device
/// 2. Resolve key mappings using `KeyLookup`
/// 3. Update state and apply mappings via `process_event`
/// 4. Inject output events to output device
///
/// Generic over `InputDevice` and `OutputDevice` traits for maximum flexibility
/// and testability. Can be used with real platform implementations or mocks.
///
/// # Type Parameters
///
/// - `I`: Input device implementation (must implement `InputDevice` trait)
/// - `O`: Output device implementation (must implement `OutputDevice` trait)
///
/// # Example
///
/// ```no_run
/// use keyrx_daemon::processor::EventProcessor;
/// use keyrx_daemon::platform::{MockInput, MockOutput};
/// use keyrx_core::config::mappings::{DeviceConfig, DeviceIdentifier};
/// use keyrx_core::runtime::KeyEvent;
/// use keyrx_core::config::KeyCode;
///
/// let input = MockInput::new(vec![KeyEvent::Press(KeyCode::A)]);
/// let output = MockOutput::new();
/// let config = DeviceConfig {
///     identifier: DeviceIdentifier { pattern: String::from("*") },
///     mappings: vec![],
/// };
///
/// let mut processor = EventProcessor::new(&config, input, output);
/// processor.run().unwrap();
/// ```
pub struct EventProcessor<I: InputDevice, O: OutputDevice> {
    /// Input device for reading keyboard events
    input: I,
    /// Output device for injecting keyboard events
    output: O,
    /// Key lookup table for O(1) mapping resolution
    lookup: KeyLookup,
    /// Runtime state (modifier and lock bits)
    state: DeviceState,
}

impl<I: InputDevice, O: OutputDevice> EventProcessor<I, O> {
    /// Creates a new event processor.
    ///
    /// Initializes the processor with the given configuration, input device,
    /// and output device. Builds the key lookup table and initializes state.
    ///
    /// # Arguments
    ///
    /// - `config`: Device configuration containing key mappings
    /// - `input`: Input device for reading events
    /// - `output`: Output device for injecting events
    ///
    /// # Returns
    ///
    /// A new `EventProcessor` ready to process events.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_daemon::processor::EventProcessor;
    /// use keyrx_daemon::platform::{MockInput, MockOutput};
    /// use keyrx_core::config::mappings::{DeviceConfig, DeviceIdentifier};
    ///
    /// let config = DeviceConfig {
    ///     identifier: DeviceIdentifier { pattern: String::from("*") },
    ///     mappings: vec![],
    /// };
    /// let input = MockInput::new(vec![]);
    /// let output = MockOutput::new();
    ///
    /// let processor = EventProcessor::new(&config, input, output);
    /// ```
    pub fn new(config: &DeviceConfig, input: I, output: O) -> Self {
        // Build lookup table from configuration
        let lookup = KeyLookup::from_device_config(config);

        // Initialize empty state (all modifiers and locks off)
        let state = DeviceState::new();

        // Log config_loaded event
        info!(
            r#"{{"timestamp":"{}","level":"INFO","service":"keyrx_daemon","event_type":"config_loaded","context":{{"mapping_count":{}}}}}"#,
            current_timestamp(),
            config.mappings.len()
        );

        Self {
            input,
            output,
            lookup,
            state,
        }
    }

    /// Processes a single event from the input device.
    ///
    /// Reads one event from the input device, processes it through the runtime
    /// (resolving mappings and updating state), and injects all output events
    /// to the output device.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Event processed successfully
    /// - `Err(ProcessorError::Input)`: Error reading from input device
    /// - `Err(ProcessorError::Output)`: Error injecting to output device
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_daemon::processor::EventProcessor;
    /// use keyrx_daemon::platform::{MockInput, MockOutput};
    /// use keyrx_core::config::mappings::{DeviceConfig, DeviceIdentifier};
    /// use keyrx_core::runtime::KeyEvent;
    /// use keyrx_core::config::KeyCode;
    ///
    /// let input = MockInput::new(vec![KeyEvent::Press(KeyCode::A)]);
    /// let output = MockOutput::new();
    /// let config = DeviceConfig {
    ///     identifier: DeviceIdentifier { pattern: String::from("*") },
    ///     mappings: vec![],
    /// };
    ///
    /// let mut processor = EventProcessor::new(&config, input, output);
    /// processor.process_one().unwrap();
    /// ```
    pub fn process_one(&mut self) -> Result<(), ProcessorError> {
        // Start latency measurement
        let start = Instant::now();

        // Read next event from input device
        let event = self.input.next_event().map_err(|e| {
            // Log platform_error for input errors (except EndOfStream)
            if !matches!(e, DeviceError::EndOfStream) {
                error!(
                    r#"{{"timestamp":"{}","level":"ERROR","service":"keyrx_daemon","event_type":"platform_error","context":{{"error":"{}","device":"input"}}}}"#,
                    current_timestamp(),
                    e
                );
            }
            e
        })?;

        // Detect state transitions by checking mapping type
        let mapping = self.lookup.find_mapping(event.keycode(), &self.state);
        let will_transition = if let Some(m) = mapping {
            use keyrx_core::config::mappings::BaseKeyMapping;
            use keyrx_core::runtime::KeyEvent;
            match (m, event) {
                (BaseKeyMapping::Modifier { modifier_id, .. }, KeyEvent::Press(_)) => {
                    Some(format!(
                        r#"{{"transition_type":"modifier_activated","modifier_id":{}}}"#,
                        modifier_id
                    ))
                }
                (BaseKeyMapping::Modifier { modifier_id, .. }, KeyEvent::Release(_)) => {
                    Some(format!(
                        r#"{{"transition_type":"modifier_deactivated","modifier_id":{}}}"#,
                        modifier_id
                    ))
                }
                (BaseKeyMapping::Lock { lock_id, .. }, KeyEvent::Press(_)) => Some(format!(
                    r#"{{"transition_type":"lock_toggled","lock_id":{}}}"#,
                    lock_id
                )),
                _ => None,
            }
        } else {
            None
        };

        // Process event through runtime
        let output_events = process_event(event, &self.lookup, &mut self.state);

        // Log state transition if it occurred
        if let Some(context) = will_transition {
            debug!(
                r#"{{"timestamp":"{}","level":"DEBUG","service":"keyrx_daemon","event_type":"state_transition","context":{}}}"#,
                current_timestamp(),
                context
            );
        }

        // Inject all output events
        for output_event in &output_events {
            self.output
                .inject_event(*output_event)
                .map_err(|e| {
                    // Log platform_error for output errors
                    error!(
                        r#"{{"timestamp":"{}","level":"ERROR","service":"keyrx_daemon","event_type":"platform_error","context":{{"error":"{}","device":"output"}}}}"#,
                        current_timestamp(),
                        e
                    );
                    ProcessorError::Output(e)
                })?;
        }

        // Calculate latency
        let latency_us = start.elapsed().as_micros() as u64;

        // Log key_processed event
        let output_keys: Vec<String> = output_events
            .iter()
            .map(|e| format!("{:?}", e.keycode()))
            .collect();
        debug!(
            r#"{{"timestamp":"{}","level":"DEBUG","service":"keyrx_daemon","event_type":"key_processed","context":{{"input_key":"{:?}","output_keys":[{}],"latency_us":{}}}}}"#,
            current_timestamp(),
            event.keycode(),
            output_keys
                .iter()
                .map(|k| format!(r#""{}""#, k))
                .collect::<Vec<_>>()
                .join(","),
            latency_us
        );

        Ok(())
    }

    /// Runs the event processing loop until end of stream.
    ///
    /// Continuously calls `process_one()` until the input device signals
    /// end of stream (`DeviceError::EndOfStream`). This is the main entry
    /// point for running the processor.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Reached end of stream normally
    /// - `Err(ProcessorError::Input)`: Input error other than EndOfStream
    /// - `Err(ProcessorError::Output)`: Output error occurred
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_daemon::processor::EventProcessor;
    /// use keyrx_daemon::platform::{MockInput, MockOutput};
    /// use keyrx_core::config::mappings::{DeviceConfig, DeviceIdentifier};
    /// use keyrx_core::runtime::KeyEvent;
    /// use keyrx_core::config::KeyCode;
    ///
    /// let input = MockInput::new(vec![
    ///     KeyEvent::Press(KeyCode::A),
    ///     KeyEvent::Release(KeyCode::A),
    /// ]);
    /// let output = MockOutput::new();
    /// let config = DeviceConfig {
    ///     identifier: DeviceIdentifier { pattern: String::from("*") },
    ///     mappings: vec![],
    /// };
    ///
    /// let mut processor = EventProcessor::new(&config, input, output);
    /// processor.run().unwrap(); // Processes both events then returns
    /// ```
    pub fn run(&mut self) -> Result<(), ProcessorError> {
        loop {
            match self.process_one() {
                Ok(()) => continue,
                Err(ProcessorError::Input(DeviceError::EndOfStream)) => {
                    // Normal termination
                    return Ok(());
                }
                Err(e) => {
                    // Propagate other errors
                    return Err(e);
                }
            }
        }
    }

    /// Returns a reference to the output device.
    ///
    /// This is primarily useful for testing, where you can inspect the
    /// events that were injected to the output device.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_daemon::processor::EventProcessor;
    /// use keyrx_daemon::platform::{MockInput, MockOutput};
    /// use keyrx_core::config::mappings::{DeviceConfig, DeviceIdentifier};
    /// use keyrx_core::runtime::KeyEvent;
    /// use keyrx_core::config::KeyCode;
    ///
    /// let input = MockInput::new(vec![KeyEvent::Press(KeyCode::A)]);
    /// let output = MockOutput::new();
    /// let config = DeviceConfig {
    ///     identifier: DeviceIdentifier { pattern: String::from("*") },
    ///     mappings: vec![],
    /// };
    ///
    /// let mut processor = EventProcessor::new(&config, input, output);
    /// processor.run().unwrap();
    ///
    /// // Access output device to verify events
    /// let events = processor.output().events();
    /// assert_eq!(events.len(), 1);
    /// ```
    pub fn output(&self) -> &O {
        &self.output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::{MockInput, MockOutput};
    use keyrx_core::config::{
        mappings::{BaseKeyMapping, DeviceConfig, DeviceIdentifier, KeyMapping},
        Condition, ConditionItem, KeyCode,
    };
    use keyrx_core::runtime::KeyEvent;
    extern crate alloc;
    use alloc::string::String;

    /// Helper to create a simple test config
    fn create_test_config(mappings: Vec<KeyMapping>) -> DeviceConfig {
        DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: String::from("*"),
            },
            mappings,
        }
    }

    #[test]
    fn test_new_creates_processor() {
        let config = create_test_config(vec![]);
        let input = MockInput::new(vec![]);
        let output = MockOutput::new();

        let _processor = EventProcessor::new(&config, input, output);
        // If we get here, constructor succeeded
    }

    #[test]
    fn test_process_one_passthrough() {
        let config = create_test_config(vec![]);
        let input = MockInput::new(vec![KeyEvent::Press(KeyCode::A)]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);
        processor.process_one().unwrap();

        // Verify passthrough: input A should produce output A
        assert_eq!(processor.output.events().len(), 1);
        assert_eq!(processor.output.events()[0], KeyEvent::Press(KeyCode::A));
    }

    #[test]
    fn test_process_one_simple_remap() {
        // A → B mapping
        let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);
        let input = MockInput::new(vec![KeyEvent::Press(KeyCode::A)]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);
        processor.process_one().unwrap();

        // Verify remap: input A should produce output B
        assert_eq!(processor.output.events().len(), 1);
        assert_eq!(processor.output.events()[0], KeyEvent::Press(KeyCode::B));
    }

    #[test]
    fn test_process_one_modifier_no_output() {
        // CapsLock → MD_00 (modifier)
        let config = create_test_config(vec![KeyMapping::modifier(KeyCode::CapsLock, 0)]);
        let input = MockInput::new(vec![KeyEvent::Press(KeyCode::CapsLock)]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);
        processor.process_one().unwrap();

        // Modifier mapping produces no output events
        assert_eq!(processor.output.events().len(), 0);

        // Verify state updated
        assert!(processor.state.is_modifier_active(0));
    }

    #[test]
    fn test_process_one_lock_no_output() {
        // ScrollLock → LK_01 (lock)
        let config = create_test_config(vec![KeyMapping::lock(KeyCode::ScrollLock, 1)]);
        let input = MockInput::new(vec![KeyEvent::Press(KeyCode::ScrollLock)]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);
        processor.process_one().unwrap();

        // Lock mapping produces no output events
        assert_eq!(processor.output.events().len(), 0);

        // Verify state toggled
        assert!(processor.state.is_lock_active(1));
    }

    #[test]
    fn test_process_one_modified_output() {
        // 1 → Shift+1 (exclamation mark)
        let config = create_test_config(vec![KeyMapping::modified_output(
            KeyCode::Num1,
            KeyCode::Num1,
            true,  // shift
            false, // ctrl
            false, // alt
            false, // win
        )]);
        let input = MockInput::new(vec![KeyEvent::Press(KeyCode::Num1)]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);
        processor.process_one().unwrap();

        // ModifiedOutput produces multiple events: Shift press, 1 press
        assert_eq!(processor.output.events().len(), 2);
        assert_eq!(
            processor.output.events()[0],
            KeyEvent::Press(KeyCode::LShift)
        );
        assert_eq!(processor.output.events()[1], KeyEvent::Press(KeyCode::Num1));
    }

    #[test]
    fn test_process_one_conditional_mapping() {
        // When MD_00 active: H → Left
        let config = create_test_config(vec![
            KeyMapping::modifier(KeyCode::CapsLock, 0),
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                }],
            ),
        ]);

        // First press CapsLock to activate MD_00
        let input = MockInput::new(vec![
            KeyEvent::Press(KeyCode::CapsLock),
            KeyEvent::Press(KeyCode::H),
        ]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);

        // Process CapsLock press (activates MD_00)
        processor.process_one().unwrap();
        assert_eq!(processor.output.events().len(), 0); // Modifier produces no output

        // Process H press (should map to Left because MD_00 is active)
        processor.process_one().unwrap();
        assert_eq!(processor.output.events().len(), 1);
        assert_eq!(processor.output.events()[0], KeyEvent::Press(KeyCode::Left));
    }

    #[test]
    fn test_run_processes_all_events() {
        let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);
        let input = MockInput::new(vec![
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::A),
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::A),
        ]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);
        processor.run().unwrap();

        // Verify all 4 events processed
        assert_eq!(processor.output.events().len(), 4);
        assert_eq!(processor.output.events()[0], KeyEvent::Press(KeyCode::B));
        assert_eq!(processor.output.events()[1], KeyEvent::Release(KeyCode::B));
        assert_eq!(processor.output.events()[2], KeyEvent::Press(KeyCode::B));
        assert_eq!(processor.output.events()[3], KeyEvent::Release(KeyCode::B));
    }

    #[test]
    fn test_run_handles_end_of_stream() {
        let config = create_test_config(vec![]);
        let input = MockInput::new(vec![KeyEvent::Press(KeyCode::A)]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);

        // run() should return Ok when reaching EndOfStream
        let result = processor.run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_empty_stream() {
        let config = create_test_config(vec![]);
        let input = MockInput::new(vec![]);
        let output = MockOutput::new();

        let mut processor = EventProcessor::new(&config, input, output);

        // Empty stream should immediately return Ok
        let result = processor.run();
        assert!(result.is_ok());
        assert_eq!(processor.output.events().len(), 0);
    }
}
