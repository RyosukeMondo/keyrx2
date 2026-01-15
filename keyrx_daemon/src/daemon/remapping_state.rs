//! Remapping state management for the event loop.
//!
//! This module contains the state needed for event processing:
//! - `KeyLookup`: O(1) key-to-mapping resolution
//! - `DeviceState`: Modifier/lock bits + tap-hold processor
//!
//! The state is maintained across events and can be reloaded on SIGHUP.

use keyrx_core::config::DeviceConfig;
use keyrx_core::runtime::{DeviceState, KeyLookup};

/// Container for remapping state.
///
/// Holds all state needed for event processing in the hot path:
/// - KeyLookup provides O(1) constant-time key→mapping resolution
/// - DeviceState tracks 255 modifiers + 255 locks + tap-hold processor
///
/// # Performance
///
/// - Key lookup: O(1), ~5ns average (HashMap with robin hood hashing)
/// - State access: O(1), direct field access
pub struct RemappingState {
    /// O(1) key-to-mapping lookup table.
    lookup: KeyLookup,
    /// Device state (modifiers, locks, tap-hold).
    state: DeviceState,
}

impl RemappingState {
    /// Creates a new remapping state from device configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Device configuration containing key mappings
    pub fn new(config: &DeviceConfig) -> Self {
        Self {
            lookup: KeyLookup::from_device_config(config),
            state: DeviceState::new(),
        }
    }

    /// Returns a reference to the key lookup table.
    #[inline]
    pub fn lookup(&self) -> &KeyLookup {
        &self.lookup
    }

    /// Returns a mutable reference to the device state.
    #[inline]
    pub fn state_mut(&mut self) -> &mut DeviceState {
        &mut self.state
    }

    /// Returns a reference to the device state.
    #[inline]
    pub fn state(&self) -> &DeviceState {
        &self.state
    }

    /// Returns both the lookup table reference and mutable state reference.
    ///
    /// This method allows borrowing the lookup and state simultaneously,
    /// which is required for `process_event()` calls. Using separate
    /// `lookup()` and `state_mut()` calls would cause a borrow conflict.
    #[inline]
    pub fn lookup_and_state_mut(&mut self) -> (&KeyLookup, &mut DeviceState) {
        (&self.lookup, &mut self.state)
    }

    /// Reloads the remapping state with new configuration.
    ///
    /// Called on SIGHUP to apply configuration changes.
    /// This creates a fresh lookup table and resets device state.
    ///
    /// # Arguments
    ///
    /// * `config` - New device configuration
    pub fn reload(&mut self, config: &DeviceConfig) {
        self.lookup = KeyLookup::from_device_config(config);
        self.state = DeviceState::new();
    }

    /// Resets only the device state (preserves lookup table).
    ///
    /// Useful for testing or recovering from stuck state.
    pub fn reset_state(&mut self) {
        self.state = DeviceState::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyrx_core::config::{DeviceIdentifier, KeyCode, KeyMapping};

    fn create_test_config() -> DeviceConfig {
        DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: "*".to_string(), // Match all devices (global)
            },
            mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
        }
    }

    #[test]
    fn test_remapping_state_new() {
        let config = create_test_config();
        let state = RemappingState::new(&config);

        // Should have created lookup table
        assert!(state
            .lookup()
            .find_mapping(KeyCode::A, state.state())
            .is_some());
    }

    #[test]
    fn test_remapping_state_reload() {
        let config = create_test_config();
        let mut state = RemappingState::new(&config);

        // Modify state
        state.state_mut().set_modifier(0);
        assert!(state.state().is_modifier_active(0));

        // Reload should reset state
        state.reload(&config);
        assert!(!state.state().is_modifier_active(0));
    }

    #[test]
    fn test_remapping_state_reset_state() {
        let config = create_test_config();
        let mut state = RemappingState::new(&config);

        // Set modifier
        state.state_mut().set_modifier(0);
        assert!(state.state().is_modifier_active(0));

        // Reset should clear modifier but preserve lookup
        state.reset_state();
        assert!(!state.state().is_modifier_active(0));
        // Lookup should still work
        assert!(state
            .lookup()
            .find_mapping(KeyCode::A, state.state())
            .is_some());
    }
}
