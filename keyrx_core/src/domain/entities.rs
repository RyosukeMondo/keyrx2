//! Domain entities for Core domain
//!
//! Entities have unique identity and lifecycle.

use crate::config::KeyCode;
use crate::runtime::KeyEvent;

/// KeyEvent entity with unique identity
///
/// Wraps the value-type KeyEvent with identity and lifecycle management.
pub struct KeyEventEntity {
    /// Unique identifier for this event
    id: u64,
    /// The actual key event
    event: KeyEvent,
    /// Timestamp when this entity was created (microseconds)
    created_at: u64,
    /// Whether this event has been processed
    processed: bool,
}

impl KeyEventEntity {
    /// Creates a new KeyEvent entity
    pub fn new(id: u64, event: KeyEvent, timestamp: u64) -> Self {
        Self {
            id,
            event,
            created_at: timestamp,
            processed: false,
        }
    }

    /// Gets the entity ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Gets the underlying event
    pub fn event(&self) -> &KeyEvent {
        &self.event
    }

    /// Gets the creation timestamp
    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    /// Checks if this event has been processed
    pub fn is_processed(&self) -> bool {
        self.processed
    }

    /// Marks this event as processed
    pub fn mark_processed(&mut self) {
        self.processed = true;
    }
}

/// Action entity representing a remapping action
///
/// Actions are the output of the remapping process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    /// The output key code
    pub key_code: KeyCode,
    /// Whether this is a press or release
    pub is_press: bool,
    /// Modifiers to apply
    pub modifiers: ModifierSet,
}

/// Set of modifiers for an action
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModifierSet {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub win: bool,
}

impl Action {
    /// Creates a simple action (no modifiers)
    pub fn simple(key_code: KeyCode, is_press: bool) -> Self {
        Self {
            key_code,
            is_press,
            modifiers: ModifierSet::default(),
        }
    }

    /// Creates an action with modifiers
    pub fn with_modifiers(
        key_code: KeyCode,
        is_press: bool,
        shift: bool,
        ctrl: bool,
        alt: bool,
        win: bool,
    ) -> Self {
        Self {
            key_code,
            is_press,
            modifiers: ModifierSet {
                shift,
                ctrl,
                alt,
                win,
            },
        }
    }

    /// Checks if this action has any modifiers
    pub fn has_modifiers(&self) -> bool {
        self.modifiers.shift || self.modifiers.ctrl || self.modifiers.alt || self.modifiers.win
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_event_entity_creation() {
        let event = KeyEvent::press(KeyCode::A).with_timestamp(1000);

        let entity = KeyEventEntity::new(1, event, 1000);

        assert_eq!(entity.id(), 1);
        assert_eq!(entity.created_at(), 1000);
        assert!(!entity.is_processed());
    }

    #[test]
    fn test_key_event_entity_processing() {
        let event = KeyEvent::press(KeyCode::A).with_timestamp(1000);

        let mut entity = KeyEventEntity::new(1, event, 1000);

        assert!(!entity.is_processed());
        entity.mark_processed();
        assert!(entity.is_processed());
    }

    #[test]
    fn test_action_simple() {
        let action = Action::simple(KeyCode::B, true);

        assert_eq!(action.key_code, KeyCode::B);
        assert!(action.is_press);
        assert!(!action.has_modifiers());
    }

    #[test]
    fn test_action_with_modifiers() {
        let action = Action::with_modifiers(KeyCode::C, true, true, true, false, false);

        assert_eq!(action.key_code, KeyCode::C);
        assert!(action.is_press);
        assert!(action.has_modifiers());
        assert!(action.modifiers.shift);
        assert!(action.modifiers.ctrl);
        assert!(!action.modifiers.alt);
    }
}
