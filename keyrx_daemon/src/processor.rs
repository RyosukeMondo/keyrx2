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

use keyrx_core::config::DeviceConfig;
use keyrx_core::runtime::event::process_event;
use keyrx_core::runtime::{DeviceState, KeyLookup};
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
        // Read next event from input device
        let event = self.input.next_event()?;

        // Process event through runtime
        let output_events = process_event(event, &self.lookup, &mut self.state);

        // Inject all output events
        for output_event in output_events {
            self.output
                .inject_event(output_event)
                .map_err(ProcessorError::Output)?;
        }

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
