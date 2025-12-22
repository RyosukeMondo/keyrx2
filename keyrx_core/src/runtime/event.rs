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
        BaseKeyMapping::ModifiedOutput { .. } => {
            // TODO: Implement in task 11
            Vec::new()
        }
    }
}
