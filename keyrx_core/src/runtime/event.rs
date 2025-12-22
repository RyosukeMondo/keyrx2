//! Keyboard event types and processing logic
//!
//! This module provides:
//! - `KeyEvent`: Type-safe keyboard event representation
//! - `process_event`: Core event processing function

extern crate alloc;
use alloc::vec::Vec;

use crate::config::KeyCode;
use crate::runtime::{DeviceState, KeyLookup};

/// Keyboard event representing a key press or release
///
/// # Example
///
/// ```rust,ignore
/// use keyrx_core::runtime::KeyEvent;
/// use keyrx_core::config::KeyCode;
///
/// let event = KeyEvent::Press(KeyCode::A);
/// assert_eq!(event.keycode(), KeyCode::A);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyEvent {
    /// Key press event
    Press(KeyCode),
    /// Key release event
    Release(KeyCode),
}

impl KeyEvent {
    /// Returns the keycode for this event
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let event = KeyEvent::Press(KeyCode::A);
    /// assert_eq!(event.keycode(), KeyCode::A);
    /// ```
    pub fn keycode(&self) -> KeyCode {
        match self {
            KeyEvent::Press(k) => *k,
            KeyEvent::Release(k) => *k,
        }
    }
}

/// Process a keyboard event through the remapping engine
///
/// Returns a vector of output events based on the mapping configuration.
/// May return:
/// - Empty vector (for modifier/lock mappings)
/// - Single event (for simple remapping or passthrough)
/// - Multiple events (for modified output sequences)
///
/// # Arguments
///
/// * `event` - Input keyboard event
/// * `lookup` - Key lookup table for mapping resolution
/// * `state` - Mutable device state for modifier/lock tracking
///
/// # Example
///
/// ```rust,ignore
/// use keyrx_core::runtime::{process_event, KeyEvent, KeyLookup, DeviceState};
///
/// let lookup = KeyLookup::from_device_config(&config);
/// let mut state = DeviceState::new();
/// let input = KeyEvent::Press(KeyCode::A);
/// let outputs = process_event(input, &lookup, &mut state);
/// ```
pub fn process_event(
    event: KeyEvent,
    lookup: &KeyLookup,
    state: &mut DeviceState,
) -> Vec<KeyEvent> {
    use crate::config::BaseKeyMapping;

    // Look up the mapping for this key
    let mapping = lookup.find_mapping(event.keycode(), state);

    // If no mapping found, pass through the original event
    let Some(mapping) = mapping else {
        return alloc::vec![event];
    };

    // Process the mapping based on its type
    match mapping {
        BaseKeyMapping::Simple { to, .. } => {
            // Simple remapping: replace keycode while preserving Press/Release
            match event {
                KeyEvent::Press(_) => alloc::vec![KeyEvent::Press(*to)],
                KeyEvent::Release(_) => alloc::vec![KeyEvent::Release(*to)],
            }
        }
        BaseKeyMapping::Modifier { modifier_id, .. } => {
            // Modifier mapping: update state, no output events
            match event {
                KeyEvent::Press(_) => {
                    state.set_modifier(*modifier_id);
                }
                KeyEvent::Release(_) => {
                    state.clear_modifier(*modifier_id);
                }
            }
            Vec::new()
        }
        BaseKeyMapping::Lock { lock_id, .. } => {
            // Lock mapping: toggle on press, ignore release, no output events
            match event {
                KeyEvent::Press(_) => {
                    state.toggle_lock(*lock_id);
                }
                KeyEvent::Release(_) => {
                    // Do nothing on release
                }
            }
            Vec::new()
        }
        BaseKeyMapping::TapHold { .. } => {
            // TODO: TapHold deferred to advanced-input-logic spec
            Vec::new()
        }
        BaseKeyMapping::ModifiedOutput {
            to,
            shift,
            ctrl,
            alt,
            win,
            ..
        } => {
            // ModifiedOutput: emit modifier presses, then key, then releases in reverse
            use crate::config::KeyCode;

            let mut events = Vec::new();

            match event {
                KeyEvent::Press(_) => {
                    // Press modifiers first, then the key
                    if *shift {
                        events.push(KeyEvent::Press(KeyCode::LShift));
                    }
                    if *ctrl {
                        events.push(KeyEvent::Press(KeyCode::LCtrl));
                    }
                    if *alt {
                        events.push(KeyEvent::Press(KeyCode::LAlt));
                    }
                    if *win {
                        events.push(KeyEvent::Press(KeyCode::LMeta));
                    }
                    events.push(KeyEvent::Press(*to));
                }
                KeyEvent::Release(_) => {
                    // Release key first, then modifiers in reverse order
                    events.push(KeyEvent::Release(*to));
                    if *win {
                        events.push(KeyEvent::Release(KeyCode::LMeta));
                    }
                    if *alt {
                        events.push(KeyEvent::Release(KeyCode::LAlt));
                    }
                    if *ctrl {
                        events.push(KeyEvent::Release(KeyCode::LCtrl));
                    }
                    if *shift {
                        events.push(KeyEvent::Release(KeyCode::LShift));
                    }
                }
            }

            events
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use crate::config::{
        BaseKeyMapping, Condition, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping,
    };
    use crate::runtime::{DeviceState, KeyLookup};
    use alloc::string::String;
    use alloc::vec;

    /// Helper to create a test DeviceConfig with given mappings
    fn create_test_config(mappings: Vec<KeyMapping>) -> DeviceConfig {
        DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: String::from("*"),
            },
            mappings,
        }
    }

    #[test]
    fn test_process_event_passthrough_no_mapping() {
        // Test passthrough: unmapped key returns original event
        let config = create_test_config(vec![]);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Test Press event
        let input_press = KeyEvent::Press(KeyCode::A);
        let output = process_event(input_press, &lookup, &mut state);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], KeyEvent::Press(KeyCode::A));

        // Test Release event
        let input_release = KeyEvent::Release(KeyCode::A);
        let output = process_event(input_release, &lookup, &mut state);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], KeyEvent::Release(KeyCode::A));
    }

    #[test]
    fn test_process_event_simple_mapping() {
        // Test Simple mapping: A → B
        let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Test Press(A) returns Press(B)
        let input_press = KeyEvent::Press(KeyCode::A);
        let output = process_event(input_press, &lookup, &mut state);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], KeyEvent::Press(KeyCode::B));

        // Test Release(A) returns Release(B)
        let input_release = KeyEvent::Release(KeyCode::A);
        let output = process_event(input_release, &lookup, &mut state);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], KeyEvent::Release(KeyCode::B));
    }

    #[test]
    fn test_process_event_modifier_mapping() {
        // Test Modifier mapping: sets state on press, clears on release, no output
        let config = create_test_config(vec![KeyMapping::modifier(KeyCode::CapsLock, 0)]);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Verify modifier is initially inactive
        assert!(!state.is_modifier_active(0));

        // Press should set modifier and return empty Vec
        let input_press = KeyEvent::Press(KeyCode::CapsLock);
        let output = process_event(input_press, &lookup, &mut state);
        assert_eq!(output.len(), 0);
        assert!(state.is_modifier_active(0));

        // Release should clear modifier and return empty Vec
        let input_release = KeyEvent::Release(KeyCode::CapsLock);
        let output = process_event(input_release, &lookup, &mut state);
        assert_eq!(output.len(), 0);
        assert!(!state.is_modifier_active(0));
    }

    #[test]
    fn test_process_event_lock_mapping() {
        // Test Lock mapping: toggles on press, no output
        let config = create_test_config(vec![KeyMapping::lock(KeyCode::ScrollLock, 1)]);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Verify lock is initially inactive
        assert!(!state.is_lock_active(1));

        // First press: toggle ON
        let input_press = KeyEvent::Press(KeyCode::ScrollLock);
        let output = process_event(input_press, &lookup, &mut state);
        assert_eq!(output.len(), 0);
        assert!(state.is_lock_active(1));

        // Release: do nothing
        let input_release = KeyEvent::Release(KeyCode::ScrollLock);
        let output = process_event(input_release, &lookup, &mut state);
        assert_eq!(output.len(), 0);
        assert!(state.is_lock_active(1)); // Still ON

        // Second press: toggle OFF
        let input_press = KeyEvent::Press(KeyCode::ScrollLock);
        let output = process_event(input_press, &lookup, &mut state);
        assert_eq!(output.len(), 0);
        assert!(!state.is_lock_active(1));

        // Release: do nothing
        let input_release = KeyEvent::Release(KeyCode::ScrollLock);
        let output = process_event(input_release, &lookup, &mut state);
        assert_eq!(output.len(), 0);
        assert!(!state.is_lock_active(1)); // Still OFF

        // Third press: toggle ON again
        let input_press = KeyEvent::Press(KeyCode::ScrollLock);
        let output = process_event(input_press, &lookup, &mut state);
        assert_eq!(output.len(), 0);
        assert!(state.is_lock_active(1));
    }

    #[test]
    fn test_process_event_modified_output_shift() {
        // Test ModifiedOutput: Shift+1 sequence
        let config = create_test_config(vec![KeyMapping::modified_output(
            KeyCode::A,
            KeyCode::Num1,
            true,  // shift
            false, // ctrl
            false, // alt
            false, // win
        )]);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Press should emit: Press(LShift), Press(Num1)
        let input_press = KeyEvent::Press(KeyCode::A);
        let output = process_event(input_press, &lookup, &mut state);
        assert_eq!(output.len(), 2);
        assert_eq!(output[0], KeyEvent::Press(KeyCode::LShift));
        assert_eq!(output[1], KeyEvent::Press(KeyCode::Num1));

        // Release should emit: Release(Num1), Release(LShift) (reverse order)
        let input_release = KeyEvent::Release(KeyCode::A);
        let output = process_event(input_release, &lookup, &mut state);
        assert_eq!(output.len(), 2);
        assert_eq!(output[0], KeyEvent::Release(KeyCode::Num1));
        assert_eq!(output[1], KeyEvent::Release(KeyCode::LShift));
    }

    #[test]
    fn test_process_event_modified_output_ctrl_alt() {
        // Test ModifiedOutput: Ctrl+Alt+C sequence
        let config = create_test_config(vec![KeyMapping::modified_output(
            KeyCode::A,
            KeyCode::C,
            false, // shift
            true,  // ctrl
            true,  // alt
            false, // win
        )]);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Press should emit: Press(LCtrl), Press(LAlt), Press(C)
        let input_press = KeyEvent::Press(KeyCode::A);
        let output = process_event(input_press, &lookup, &mut state);
        assert_eq!(output.len(), 3);
        assert_eq!(output[0], KeyEvent::Press(KeyCode::LCtrl));
        assert_eq!(output[1], KeyEvent::Press(KeyCode::LAlt));
        assert_eq!(output[2], KeyEvent::Press(KeyCode::C));

        // Release should emit in reverse: Release(C), Release(LAlt), Release(LCtrl)
        let input_release = KeyEvent::Release(KeyCode::A);
        let output = process_event(input_release, &lookup, &mut state);
        assert_eq!(output.len(), 3);
        assert_eq!(output[0], KeyEvent::Release(KeyCode::C));
        assert_eq!(output[1], KeyEvent::Release(KeyCode::LAlt));
        assert_eq!(output[2], KeyEvent::Release(KeyCode::LCtrl));
    }

    #[test]
    fn test_process_event_modified_output_all_modifiers() {
        // Test ModifiedOutput: Shift+Ctrl+Alt+Win+Key sequence
        let config = create_test_config(vec![KeyMapping::modified_output(
            KeyCode::A,
            KeyCode::Z,
            true, // shift
            true, // ctrl
            true, // alt
            true, // win
        )]);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Press should emit all modifiers then key
        let input_press = KeyEvent::Press(KeyCode::A);
        let output = process_event(input_press, &lookup, &mut state);
        assert_eq!(output.len(), 5);
        assert_eq!(output[0], KeyEvent::Press(KeyCode::LShift));
        assert_eq!(output[1], KeyEvent::Press(KeyCode::LCtrl));
        assert_eq!(output[2], KeyEvent::Press(KeyCode::LAlt));
        assert_eq!(output[3], KeyEvent::Press(KeyCode::LMeta));
        assert_eq!(output[4], KeyEvent::Press(KeyCode::Z));

        // Release should emit in reverse order
        let input_release = KeyEvent::Release(KeyCode::A);
        let output = process_event(input_release, &lookup, &mut state);
        assert_eq!(output.len(), 5);
        assert_eq!(output[0], KeyEvent::Release(KeyCode::Z));
        assert_eq!(output[1], KeyEvent::Release(KeyCode::LMeta));
        assert_eq!(output[2], KeyEvent::Release(KeyCode::LAlt));
        assert_eq!(output[3], KeyEvent::Release(KeyCode::LCtrl));
        assert_eq!(output[4], KeyEvent::Release(KeyCode::LShift));
    }

    #[test]
    fn test_process_event_conditional_mapping_true() {
        // Test Conditional mapping: when modifier active, apply conditional mapping
        let config = create_test_config(vec![
            // CapsLock activates MD_00
            KeyMapping::modifier(KeyCode::CapsLock, 0),
            // when(MD_00): H → Left
            KeyMapping::conditional(
                Condition::ModifierActive(0),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                }],
            ),
        ]);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Activate modifier
        state.set_modifier(0);

        // With modifier active, H should map to Left
        let input_press = KeyEvent::Press(KeyCode::H);
        let output = process_event(input_press, &lookup, &mut state);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], KeyEvent::Press(KeyCode::Left));

        let input_release = KeyEvent::Release(KeyCode::H);
        let output = process_event(input_release, &lookup, &mut state);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], KeyEvent::Release(KeyCode::Left));
    }

    #[test]
    fn test_process_event_conditional_mapping_false() {
        // Test Conditional mapping: when modifier NOT active, passthrough
        let config = create_test_config(vec![
            // CapsLock activates MD_00
            KeyMapping::modifier(KeyCode::CapsLock, 0),
            // when(MD_00): H → Left
            KeyMapping::conditional(
                Condition::ModifierActive(0),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                }],
            ),
        ]);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Modifier is NOT active, H should passthrough
        let input_press = KeyEvent::Press(KeyCode::H);
        let output = process_event(input_press, &lookup, &mut state);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], KeyEvent::Press(KeyCode::H)); // Passthrough

        let input_release = KeyEvent::Release(KeyCode::H);
        let output = process_event(input_release, &lookup, &mut state);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], KeyEvent::Release(KeyCode::H)); // Passthrough
    }

    #[test]
    fn test_process_event_conditional_with_unconditional_fallback() {
        // Test conditional with fallback: if condition false, use unconditional mapping
        let config = create_test_config(vec![
            // when(MD_00): H → Left (conditional)
            KeyMapping::conditional(
                Condition::ModifierActive(0),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                }],
            ),
            // H → Down (unconditional fallback)
            KeyMapping::simple(KeyCode::H, KeyCode::Down),
        ]);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Modifier NOT active: should use unconditional mapping (H → Down)
        let input_press = KeyEvent::Press(KeyCode::H);
        let output = process_event(input_press, &lookup, &mut state);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], KeyEvent::Press(KeyCode::Down));

        // Activate modifier
        state.set_modifier(0);

        // Modifier active: should use conditional mapping (H → Left)
        let input_press = KeyEvent::Press(KeyCode::H);
        let output = process_event(input_press, &lookup, &mut state);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], KeyEvent::Press(KeyCode::Left));
    }

    #[test]
    fn test_process_event_vim_navigation_layer() {
        // Test realistic Vim navigation: CapsLock as layer, HJKL → arrow keys
        let config = create_test_config(vec![
            // CapsLock activates MD_00
            KeyMapping::modifier(KeyCode::CapsLock, 0),
            // when(MD_00): H → Left
            KeyMapping::conditional(
                Condition::ModifierActive(0),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                }],
            ),
            // when(MD_00): J → Down
            KeyMapping::conditional(
                Condition::ModifierActive(0),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::J,
                    to: KeyCode::Down,
                }],
            ),
            // when(MD_00): K → Up
            KeyMapping::conditional(
                Condition::ModifierActive(0),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::K,
                    to: KeyCode::Up,
                }],
            ),
            // when(MD_00): L → Right
            KeyMapping::conditional(
                Condition::ModifierActive(0),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::L,
                    to: KeyCode::Right,
                }],
            ),
        ]);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Press CapsLock to activate layer
        let _ = process_event(KeyEvent::Press(KeyCode::CapsLock), &lookup, &mut state);
        assert!(state.is_modifier_active(0));

        // Test H → Left
        let output = process_event(KeyEvent::Press(KeyCode::H), &lookup, &mut state);
        assert_eq!(output[0], KeyEvent::Press(KeyCode::Left));

        // Test J → Down
        let output = process_event(KeyEvent::Press(KeyCode::J), &lookup, &mut state);
        assert_eq!(output[0], KeyEvent::Press(KeyCode::Down));

        // Test K → Up
        let output = process_event(KeyEvent::Press(KeyCode::K), &lookup, &mut state);
        assert_eq!(output[0], KeyEvent::Press(KeyCode::Up));

        // Test L → Right
        let output = process_event(KeyEvent::Press(KeyCode::L), &lookup, &mut state);
        assert_eq!(output[0], KeyEvent::Press(KeyCode::Right));

        // Release CapsLock to deactivate layer
        let _ = process_event(KeyEvent::Release(KeyCode::CapsLock), &lookup, &mut state);
        assert!(!state.is_modifier_active(0));

        // H should now passthrough (no mapping when layer inactive)
        let output = process_event(KeyEvent::Press(KeyCode::H), &lookup, &mut state);
        assert_eq!(output[0], KeyEvent::Press(KeyCode::H));
    }

    #[test]
    fn test_keyevent_keycode_helper() {
        // Test KeyEvent::keycode() helper method
        let press = KeyEvent::Press(KeyCode::A);
        assert_eq!(press.keycode(), KeyCode::A);

        let release = KeyEvent::Release(KeyCode::B);
        assert_eq!(release.keycode(), KeyCode::B);
    }

    #[test]
    fn test_keyevent_derives() {
        use alloc::format;

        // Test that KeyEvent has all expected derives
        let event1 = KeyEvent::Press(KeyCode::A);
        let event2 = KeyEvent::Press(KeyCode::A);
        let event3 = KeyEvent::Release(KeyCode::A);

        // Test Clone (Copy trait)
        let cloned = event1;
        assert_eq!(cloned, event1);

        // Test PartialEq and Eq
        assert_eq!(event1, event2);
        assert_ne!(event1, event3);

        // Test Debug (should not panic)
        let _ = format!("{:?}", event1);

        // Test Hash - verify that equal events produce equal hashes
        use core::hash::{Hash, Hasher};

        // Simple test hasher that accumulates hash values
        struct TestHasher {
            value: u64,
        }

        impl Hasher for TestHasher {
            fn finish(&self) -> u64 {
                self.value
            }

            fn write(&mut self, bytes: &[u8]) {
                for &byte in bytes {
                    self.value = self.value.wrapping_mul(31).wrapping_add(byte as u64);
                }
            }
        }

        let mut hasher1 = TestHasher { value: 0 };
        event1.hash(&mut hasher1);

        let mut hasher2 = TestHasher { value: 0 };
        event2.hash(&mut hasher2);

        // Equal values should have equal hashes
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    // Property-based tests using proptest
    #[cfg(test)]
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        // Strategy to generate arbitrary KeyCode values (A-Z range for simplicity)
        fn keycode_strategy() -> impl Strategy<Value = KeyCode> {
            prop::sample::select(vec![
                KeyCode::A,
                KeyCode::B,
                KeyCode::C,
                KeyCode::D,
                KeyCode::E,
                KeyCode::F,
                KeyCode::G,
                KeyCode::H,
                KeyCode::I,
                KeyCode::J,
                KeyCode::K,
                KeyCode::L,
                KeyCode::M,
                KeyCode::N,
                KeyCode::O,
                KeyCode::P,
                KeyCode::Q,
                KeyCode::R,
                KeyCode::S,
                KeyCode::T,
                KeyCode::U,
                KeyCode::V,
                KeyCode::W,
                KeyCode::X,
                KeyCode::Y,
                KeyCode::Z,
            ])
        }

        // Strategy to generate arbitrary KeyEvent values
        fn keyevent_strategy() -> impl Strategy<Value = KeyEvent> {
            (keycode_strategy(), prop::bool::ANY).prop_map(|(keycode, is_press)| {
                if is_press {
                    KeyEvent::Press(keycode)
                } else {
                    KeyEvent::Release(keycode)
                }
            })
        }

        proptest! {
            /// Property test: No event loss for Simple mappings
            ///
            /// Invariant: For Simple mappings (1:1), the number of input events
            /// must equal the number of output events. Every input event produces
            /// exactly one output event, no events are lost or duplicated.
            #[test]
            fn prop_no_event_loss(events in prop::collection::vec(keyevent_strategy(), 1..100)) {
                // Create config with Simple mapping A → B
                let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);
                let lookup = KeyLookup::from_device_config(&config);
                let mut state = DeviceState::new();

                // Process all events
                let mut total_output_count = 0;
                for event in &events {
                    let outputs = process_event(*event, &lookup, &mut state);
                    total_output_count += outputs.len();
                }

                // For Simple mapping (1:1), input count must equal output count
                // (passthrough also produces 1 output per 1 input)
                prop_assert_eq!(events.len(), total_output_count,
                    "Event loss detected: {} inputs produced {} outputs",
                    events.len(), total_output_count);
            }

            /// Property test: Deterministic execution
            ///
            /// Invariant: Processing the same event sequence twice with the same
            /// initial state must produce identical outputs. The event processing
            /// is deterministic and has no hidden state or randomness.
            #[test]
            fn prop_deterministic(events in prop::collection::vec(keyevent_strategy(), 1..50)) {
                // Create config with Simple mapping A → B
                let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);
                let lookup = KeyLookup::from_device_config(&config);

                // First run
                let mut state1 = DeviceState::new();
                let mut outputs1 = Vec::new();
                for event in &events {
                    let result = process_event(*event, &lookup, &mut state1);
                    outputs1.extend(result);
                }

                // Second run (identical initial state)
                let mut state2 = DeviceState::new();
                let mut outputs2 = Vec::new();
                for event in &events {
                    let result = process_event(*event, &lookup, &mut state2);
                    outputs2.extend(result);
                }

                // Outputs must be byte-for-byte identical
                prop_assert_eq!(outputs1, outputs2,
                    "Non-deterministic behavior: same inputs produced different outputs");

                // Final states must also be identical
                // (We can't directly compare DeviceState since it doesn't implement PartialEq,
                // but we can verify that the outputs are identical, which is sufficient for determinism)
            }

            /// Property test: Modifier events produce no output
            ///
            /// Invariant: Modifier mappings never produce output events, they only
            /// update internal state. This ensures modifiers are purely stateful.
            #[test]
            fn prop_modifier_no_output(events in prop::collection::vec(keyevent_strategy(), 1..50)) {
                // Create config with Modifier mapping: A activates MD_00
                let config = create_test_config(vec![KeyMapping::modifier(KeyCode::A, 0)]);
                let lookup = KeyLookup::from_device_config(&config);
                let mut state = DeviceState::new();

                // Process all events
                for event in &events {
                    let outputs = process_event(*event, &lookup, &mut state);

                    // If event is for key A (modifier key), output must be empty
                    if event.keycode() == KeyCode::A {
                        prop_assert!(outputs.is_empty(),
                            "Modifier mapping produced output: {:?}", outputs);
                    }
                }
            }

            /// Property test: Lock events produce no output
            ///
            /// Invariant: Lock mappings never produce output events, they only
            /// toggle internal state. This ensures locks are purely stateful.
            #[test]
            fn prop_lock_no_output(events in prop::collection::vec(keyevent_strategy(), 1..50)) {
                // Create config with Lock mapping: A activates LK_00
                let config = create_test_config(vec![KeyMapping::lock(KeyCode::A, 0)]);
                let lookup = KeyLookup::from_device_config(&config);
                let mut state = DeviceState::new();

                // Process all events
                for event in &events {
                    let outputs = process_event(*event, &lookup, &mut state);

                    // If event is for key A (lock key), output must be empty
                    if event.keycode() == KeyCode::A {
                        prop_assert!(outputs.is_empty(),
                            "Lock mapping produced output: {:?}", outputs);
                    }
                }
            }

            /// Property test: Passthrough preserves event type
            ///
            /// Invariant: When no mapping exists, the output event must have the
            /// same keycode and type (Press/Release) as the input event.
            #[test]
            fn prop_passthrough_preserves_event(events in prop::collection::vec(keyevent_strategy(), 1..50)) {
                // Create empty config (all events passthrough)
                let config = create_test_config(vec![]);
                let lookup = KeyLookup::from_device_config(&config);
                let mut state = DeviceState::new();

                // Process all events
                for event in &events {
                    let outputs = process_event(*event, &lookup, &mut state);

                    // Must produce exactly one output
                    prop_assert_eq!(outputs.len(), 1,
                        "Passthrough produced {} outputs for 1 input", outputs.len());

                    // Output must be identical to input
                    prop_assert_eq!(outputs[0], *event,
                        "Passthrough modified event: {:?} became {:?}", event, outputs[0]);
                }
            }

            /// Property test: Simple mapping preserves event type
            ///
            /// Invariant: Simple mappings change the keycode but preserve the
            /// event type (Press stays Press, Release stays Release).
            #[test]
            fn prop_simple_preserves_type(events in prop::collection::vec(keyevent_strategy(), 1..50)) {
                // Create config with Simple mapping A → B
                let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);
                let lookup = KeyLookup::from_device_config(&config);
                let mut state = DeviceState::new();

                // Process all events
                for event in &events {
                    let outputs = process_event(*event, &lookup, &mut state);

                    // Must produce exactly one output (simple is 1:1)
                    prop_assert_eq!(outputs.len(), 1,
                        "Simple mapping produced {} outputs for 1 input", outputs.len());

                    // Verify event type is preserved
                    match (event, &outputs[0]) {
                        (KeyEvent::Press(_), KeyEvent::Press(_)) => {
                            // Press → Press: OK
                        }
                        (KeyEvent::Release(_), KeyEvent::Release(_)) => {
                            // Release → Release: OK
                        }
                        _ => {
                            return Err(proptest::test_runner::TestCaseError::fail(
                                alloc::format!("Event type not preserved: {:?} became {:?}", event, outputs[0])
                            ));
                        }
                    }
                }
            }
        }
    }
}
