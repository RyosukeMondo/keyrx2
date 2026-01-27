//! Domain services for Core domain
//!
//! Services contain business logic that doesn't naturally fit in entities or value objects.

use alloc::vec::Vec;

use crate::config::{ConfigRoot, KeyCode};
use crate::runtime::{KeyEvent, KeyEventType};

use super::aggregates::{KeyMappingAggregate, StateAggregate};
use super::entities::Action;
use super::events::{DomainEvent, DomainEventBus};
use super::DomainError;

/// Event processor service
///
/// Processes keyboard events through the remapping logic.
pub struct EventProcessorService {
    /// Event bus for publishing domain events
    event_bus: DomainEventBus,
}

impl EventProcessorService {
    /// Creates a new event processor
    pub fn new() -> Self {
        Self {
            event_bus: DomainEventBus::new(),
        }
    }

    /// Processes a key event and produces actions
    pub fn process_event(
        &mut self,
        event: &KeyEvent,
        mappings: &[KeyMappingAggregate],
        state: &StateAggregate,
    ) -> Result<Vec<Action>, DomainError> {
        // Publish domain event
        match event.event_type() {
            KeyEventType::Press => {
                self.event_bus.publish(DomainEvent::KeyPressed {
                    key_code: event.keycode(),
                    timestamp_us: event.timestamp_us(),
                });
            }
            KeyEventType::Release => {
                self.event_bus.publish(DomainEvent::KeyReleased {
                    key_code: event.keycode(),
                    timestamp_us: event.timestamp_us(),
                });
            }
        }

        // Find applicable mappings
        let mut actions = Vec::new();

        for mapping in mappings {
            if mapping.input() == event.keycode() && mapping.applies_to_state(state.state()) {
                // Apply mapping
                let action = self.apply_mapping(mapping, event)?;
                let output_key = action.key_code;
                actions.push(action);

                self.event_bus.publish(DomainEvent::MappingApplied {
                    input: event.keycode(),
                    output: output_key,
                    timestamp_us: event.timestamp_us(),
                });

                break; // Only first matching mapping applies
            }
        }

        // If no mapping found, pass through
        if actions.is_empty() {
            actions.push(Action::simple(
                event.keycode(),
                event.event_type() == KeyEventType::Press,
            ));
        }

        Ok(actions)
    }

    /// Applies a specific mapping
    fn apply_mapping(
        &self,
        _mapping: &KeyMappingAggregate,
        _event: &KeyEvent,
    ) -> Result<Action, DomainError> {
        // Simplified for now - real implementation would handle all mapping types
        Ok(Action::simple(KeyCode::B, true))
    }

    /// Gets the event bus for consuming events
    pub fn event_bus(&self) -> &DomainEventBus {
        &self.event_bus
    }

    /// Drains events from the bus
    pub fn drain_events(&mut self) -> Vec<DomainEvent> {
        self.event_bus.drain()
    }
}

impl Default for EventProcessorService {
    fn default() -> Self {
        Self::new()
    }
}

/// State machine service
///
/// Manages state transitions according to domain rules.
pub struct StateMachineService {
    /// Current state
    state: StateAggregate,
    /// Event bus for state change events
    event_bus: DomainEventBus,
}

impl StateMachineService {
    /// Creates a new state machine
    pub fn new() -> Self {
        Self {
            state: StateAggregate::new(),
            event_bus: DomainEventBus::new(),
        }
    }

    /// Gets the current state
    pub fn state(&self) -> &StateAggregate {
        &self.state
    }

    /// Transitions to a new state by setting a bit
    pub fn set_bit(&mut self, bit: usize, timestamp_us: u64) -> Result<(), DomainError> {
        self.state.set_bit(bit)?;

        self.event_bus.publish(DomainEvent::StateChanged {
            bit,
            new_value: true,
            timestamp_us,
        });

        Ok(())
    }

    /// Transitions by clearing a bit
    pub fn clear_bit(&mut self, bit: usize, timestamp_us: u64) -> Result<(), DomainError> {
        self.state.clear_bit(bit)?;

        self.event_bus.publish(DomainEvent::StateChanged {
            bit,
            new_value: false,
            timestamp_us,
        });

        Ok(())
    }

    /// Toggles a bit (for locks)
    pub fn toggle_bit(&mut self, bit: usize, timestamp_us: u64) -> Result<(), DomainError> {
        let new_value = !self.state.is_bit_set(bit);
        self.state.toggle_bit(bit)?;

        self.event_bus.publish(DomainEvent::StateChanged {
            bit,
            new_value,
            timestamp_us,
        });

        Ok(())
    }

    /// Resets the state machine
    pub fn reset(&mut self, timestamp_us: u64) {
        self.state.reset();

        // Publish reset event for all bits
        for bit in 0..255 {
            self.event_bus.publish(DomainEvent::StateChanged {
                bit,
                new_value: false,
                timestamp_us,
            });
        }
    }

    /// Drains events from the bus
    pub fn drain_events(&mut self) -> Vec<DomainEvent> {
        self.event_bus.drain()
    }
}

impl Default for StateMachineService {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration validation service
pub struct ConfigValidationService;

impl ConfigValidationService {
    /// Validates a configuration against domain rules
    pub fn validate(config: &ConfigRoot) -> Result<(), DomainError> {
        // Validate version
        if config.version.major == 0 && config.version.minor == 0 {
            return Err(DomainError::ConstraintViolation(
                "Invalid version 0.0.x".into(),
            ));
        }

        // Validate devices exist
        if config.devices.is_empty() {
            return Err(DomainError::ConstraintViolation(
                "At least one device required".into(),
            ));
        }

        // Validate each device
        for device in &config.devices {
            if device.identifier.pattern.is_empty() {
                return Err(DomainError::ConstraintViolation(
                    "Device pattern cannot be empty".into(),
                ));
            }

            if device.mappings.is_empty() {
                return Err(DomainError::ConstraintViolation(
                    "Device must have at least one mapping".into(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_event_processor_service() {
        let mut service = EventProcessorService::new();

        let event = KeyEvent::press(KeyCode::A).with_timestamp(1000);

        let mappings = vec![];
        let state = StateAggregate::new();

        let actions = service.process_event(&event, &mappings, &state).unwrap();

        // Should pass through if no mapping
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].key_code, KeyCode::A);

        // Should have published event
        let events = service.drain_events();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_state_machine_service() {
        let mut service = StateMachineService::new();

        // Set bit
        service.set_bit(10, 1000).unwrap();
        assert!(service.state().is_bit_set(10));

        // Clear bit
        service.clear_bit(10, 2000).unwrap();
        assert!(!service.state().is_bit_set(10));

        // Toggle bit
        service.toggle_bit(20, 3000).unwrap();
        assert!(service.state().is_bit_set(20));

        service.toggle_bit(20, 4000).unwrap();
        assert!(!service.state().is_bit_set(20));

        // Check events
        let events = service.drain_events();
        assert_eq!(events.len(), 4);
    }

    #[test]
    fn test_config_validation() {
        use crate::config::{DeviceConfig, DeviceIdentifier, Metadata, Version};

        let invalid_config = ConfigRoot {
            version: Version {
                major: 0,
                minor: 0,
                patch: 0,
            },
            devices: vec![],
            metadata: Metadata {
                compilation_timestamp: 0,
                compiler_version: "test".into(),
                source_hash: "test".into(),
            },
        };

        assert!(ConfigValidationService::validate(&invalid_config).is_err());

        let valid_config = ConfigRoot {
            version: Version::current(),
            devices: vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: ".*".into(),
                },
                mappings: vec![crate::config::KeyMapping::simple(KeyCode::A, KeyCode::B)],
            }],
            metadata: Metadata {
                compilation_timestamp: 0,
                compiler_version: "test".into(),
                source_hash: "test".into(),
            },
        };

        assert!(ConfigValidationService::validate(&valid_config).is_ok());
    }
}
