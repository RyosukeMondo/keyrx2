//! Domain aggregates for Core domain
//!
//! Aggregates are clusters of domain objects that can be treated as a single unit.
//! They have a root entity and enforce consistency boundaries.

use alloc::string::String;
use alloc::vec::Vec;

use crate::config::{BaseKeyMapping, Condition, KeyCode, KeyMapping};
use crate::runtime::DeviceState;

use super::DomainError;

/// KeyMapping aggregate root
///
/// Encapsulates the mapping logic and ensures consistency of mapping rules.
/// This is an aggregate because it maintains invariants across input, conditions,
/// and output mappings.
pub struct KeyMappingAggregate {
    /// Input key code
    input: KeyCode,
    /// Conditions that must be met for this mapping to apply
    conditions: Vec<Condition>,
    /// The mapping to apply (one of 5 base types)
    mapping: KeyMapping,
    /// Whether this mapping is currently active
    active: bool,
}

impl KeyMappingAggregate {
    /// Creates a new KeyMapping aggregate
    pub fn new(input: KeyCode, conditions: Vec<Condition>, mapping: KeyMapping) -> Self {
        Self {
            input,
            conditions,
            mapping,
            active: true,
        }
    }

    /// Gets the input key code
    pub fn input(&self) -> KeyCode {
        self.input
    }

    /// Gets the conditions
    pub fn conditions(&self) -> &[Condition] {
        &self.conditions
    }

    /// Gets the mapping
    pub fn mapping(&self) -> &KeyMapping {
        &self.mapping
    }

    /// Checks if this mapping is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Activates this mapping
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Deactivates this mapping
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Evaluates whether this mapping applies given the current state
    pub fn applies_to_state(&self, state: &DeviceState) -> bool {
        if !self.active {
            return false;
        }

        // Check all conditions
        self.conditions.iter().all(|condition| {
            state.evaluate_condition(condition)
        })
    }

    /// Validates this mapping against domain rules
    pub fn validate(&self) -> Result<(), DomainError> {
        // Validate based on mapping type
        match &self.mapping {
            KeyMapping::Base(base) => match base {
                BaseKeyMapping::Simple { from, to } => {
                    if from == to {
                        return Err(DomainError::ConstraintViolation(
                            "Simple mapping cannot map key to itself".into(),
                        ));
                    }
                }
                BaseKeyMapping::TapHold { threshold_ms, .. } => {
                    if *threshold_ms == 0 {
                        return Err(DomainError::ConstraintViolation(
                            "TapHold threshold must be > 0".into(),
                        ));
                    }
                }
                _ => {}
            },
            KeyMapping::Conditional { .. } => {
                // Conditional mappings validated separately
            }
        }

        Ok(())
    }
}

/// State aggregate root
///
/// Encapsulates the device state (255 bits for modifiers/locks) and enforces state transition rules.
pub struct StateAggregate {
    /// The device state (255 bits for modifiers, locks)
    state: DeviceState,
    /// Version counter for optimistic locking
    version: u64,
}

impl StateAggregate {
    /// Creates a new StateAggregate with zero state
    pub fn new() -> Self {
        Self {
            state: DeviceState::new(),
            version: 0,
        }
    }

    /// Creates a StateAggregate from an existing state
    pub fn from_state(state: DeviceState) -> Self {
        Self { state, version: 0 }
    }

    /// Gets the current state
    pub fn state(&self) -> &DeviceState {
        &self.state
    }

    /// Gets a mutable reference to the state
    pub fn state_mut(&mut self) -> &mut DeviceState {
        &mut self.state
    }

    /// Gets the version for optimistic locking
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Sets a modifier bit in the state
    pub fn set_bit(&mut self, bit: usize) -> Result<(), DomainError> {
        if bit >= 255 {
            return Err(DomainError::ConstraintViolation(String::from(
                "Bit index out of range (max 254)",
            )));
        }

        if !self.state.set_modifier(bit as u8) {
            return Err(DomainError::ConstraintViolation(String::from(
                "Failed to set modifier bit",
            )));
        }
        self.version += 1;
        Ok(())
    }

    /// Clears a modifier bit in the state
    pub fn clear_bit(&mut self, bit: usize) -> Result<(), DomainError> {
        if bit >= 255 {
            return Err(DomainError::ConstraintViolation(String::from(
                "Bit index out of range (max 254)",
            )));
        }

        if !self.state.clear_modifier(bit as u8) {
            return Err(DomainError::ConstraintViolation(String::from(
                "Failed to clear modifier bit",
            )));
        }
        self.version += 1;
        Ok(())
    }

    /// Toggles a lock bit in the state
    pub fn toggle_bit(&mut self, bit: usize) -> Result<(), DomainError> {
        if bit >= 255 {
            return Err(DomainError::ConstraintViolation(String::from(
                "Bit index out of range (max 254)",
            )));
        }

        if !self.state.toggle_lock(bit as u8) {
            return Err(DomainError::ConstraintViolation(String::from(
                "Failed to toggle lock bit",
            )));
        }
        self.version += 1;
        Ok(())
    }

    /// Checks if a bit is set (checks both modifiers and locks)
    pub fn is_bit_set(&self, bit: usize) -> bool {
        if bit >= 255 {
            return false;
        }
        // Check both modifiers and locks
        self.state.is_modifier_active(bit as u8) || self.state.is_lock_active(bit as u8)
    }

    /// Resets the state to zero
    pub fn reset(&mut self) {
        self.state = DeviceState::new();
        self.version += 1;
    }
}

impl Default for StateAggregate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_key_mapping_aggregate_creation() {
        let mapping = KeyMappingAggregate::new(
            KeyCode::A,
            vec![],
            KeyMapping::simple(KeyCode::A, KeyCode::B),
        );

        assert_eq!(mapping.input(), KeyCode::A);
        assert!(mapping.is_active());
        assert_eq!(mapping.conditions().len(), 0);
    }

    #[test]
    fn test_key_mapping_aggregate_validation() {
        // Valid mapping
        let mapping = KeyMappingAggregate::new(
            KeyCode::A,
            vec![],
            KeyMapping::simple(KeyCode::A, KeyCode::B),
        );
        assert!(mapping.validate().is_ok());

        // Invalid mapping (A -> A)
        let invalid_mapping = KeyMappingAggregate::new(
            KeyCode::A,
            vec![],
            KeyMapping::simple(KeyCode::A, KeyCode::A),
        );
        assert!(invalid_mapping.validate().is_err());
    }

    #[test]
    fn test_state_aggregate_bit_operations() {
        let mut state = StateAggregate::new();

        // Set bit
        assert!(state.set_bit(10).is_ok());
        assert!(state.is_bit_set(10));
        assert_eq!(state.version(), 1);

        // Clear bit
        assert!(state.clear_bit(10).is_ok());
        assert!(!state.is_bit_set(10));
        assert_eq!(state.version(), 2);

        // Toggle bit
        assert!(state.toggle_bit(20).is_ok());
        assert!(state.is_bit_set(20));
        assert_eq!(state.version(), 3);

        assert!(state.toggle_bit(20).is_ok());
        assert!(!state.is_bit_set(20));
        assert_eq!(state.version(), 4);
    }

    #[test]
    fn test_state_aggregate_bounds_checking() {
        let mut state = StateAggregate::new();

        // Out of bounds
        assert!(state.set_bit(255).is_err());
        assert!(state.clear_bit(300).is_err());
    }

    #[test]
    fn test_state_aggregate_reset() {
        let mut state = StateAggregate::new();

        state.set_bit(10).unwrap();
        state.set_bit(20).unwrap();
        assert!(state.is_bit_set(10));
        assert!(state.is_bit_set(20));

        state.reset();
        assert!(!state.is_bit_set(10));
        assert!(!state.is_bit_set(20));
    }
}
