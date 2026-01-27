//! Domain events for Core domain
//!
//! Domain events represent things that have happened in the domain.

use alloc::vec::Vec;

use crate::config::KeyCode;

/// Domain event for Core domain
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    /// A key was pressed
    KeyPressed {
        key_code: KeyCode,
        timestamp_us: u64,
    },
    /// A key was released
    KeyReleased {
        key_code: KeyCode,
        timestamp_us: u64,
    },
    /// A mapping was applied
    MappingApplied {
        input: KeyCode,
        output: KeyCode,
        timestamp_us: u64,
    },
    /// State was changed
    StateChanged {
        bit: usize,
        new_value: bool,
        timestamp_us: u64,
    },
    /// Configuration was loaded
    ConfigurationLoaded { timestamp_us: u64 },
    /// Configuration was reloaded
    ConfigurationReloaded { timestamp_us: u64 },
}

impl DomainEvent {
    /// Gets the timestamp of this event
    pub fn timestamp(&self) -> u64 {
        match self {
            Self::KeyPressed { timestamp_us, .. } => *timestamp_us,
            Self::KeyReleased { timestamp_us, .. } => *timestamp_us,
            Self::MappingApplied { timestamp_us, .. } => *timestamp_us,
            Self::StateChanged { timestamp_us, .. } => *timestamp_us,
            Self::ConfigurationLoaded { timestamp_us } => *timestamp_us,
            Self::ConfigurationReloaded { timestamp_us } => *timestamp_us,
        }
    }

    /// Gets a human-readable name for this event type
    pub fn event_type_name(&self) -> &'static str {
        match self {
            Self::KeyPressed { .. } => "KeyPressed",
            Self::KeyReleased { .. } => "KeyReleased",
            Self::MappingApplied { .. } => "MappingApplied",
            Self::StateChanged { .. } => "StateChanged",
            Self::ConfigurationLoaded { .. } => "ConfigurationLoaded",
            Self::ConfigurationReloaded { .. } => "ConfigurationReloaded",
        }
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
        let event = DomainEvent::KeyPressed {
            key_code: KeyCode::A,
            timestamp_us: 1000,
        };
        assert_eq!(event.timestamp(), 1000);

        let event = DomainEvent::ConfigurationLoaded { timestamp_us: 2000 };
        assert_eq!(event.timestamp(), 2000);
    }

    #[test]
    fn test_domain_event_type_name() {
        let event = DomainEvent::KeyPressed {
            key_code: KeyCode::A,
            timestamp_us: 1000,
        };
        assert_eq!(event.event_type_name(), "KeyPressed");

        let event = DomainEvent::MappingApplied {
            input: KeyCode::A,
            output: KeyCode::B,
            timestamp_us: 1000,
        };
        assert_eq!(event.event_type_name(), "MappingApplied");
    }

    #[test]
    fn test_event_bus() {
        let mut bus = DomainEventBus::new();

        bus.publish(DomainEvent::KeyPressed {
            key_code: KeyCode::A,
            timestamp_us: 1000,
        });
        bus.publish(DomainEvent::KeyReleased {
            key_code: KeyCode::A,
            timestamp_us: 2000,
        });

        assert_eq!(bus.events().len(), 2);

        let events = bus.drain();
        assert_eq!(events.len(), 2);
        assert_eq!(bus.events().len(), 0);
    }
}
