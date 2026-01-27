//! Domain events for Daemon domain
//!
//! Domain events represent things that have happened in the domain.

use super::value_objects::{DeviceSerialVO, ProfileNameVO};

/// Domain event for Daemon domain
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    /// Device connected
    DeviceConnected {
        serial: DeviceSerialVO,
        device_name: std::string::String,
        timestamp_us: u64,
    },
    /// Device disconnected
    DeviceDisconnected {
        serial: DeviceSerialVO,
        timestamp_us: u64,
    },
    /// Device enabled
    DeviceEnabled {
        serial: DeviceSerialVO,
        timestamp_us: u64,
    },
    /// Device disabled
    DeviceDisabled {
        serial: DeviceSerialVO,
        timestamp_us: u64,
    },
    /// Profile activated
    ProfileActivated {
        profile: ProfileNameVO,
        timestamp_us: u64,
    },
    /// Profile deactivated
    ProfileDeactivated {
        profile: ProfileNameVO,
        timestamp_us: u64,
    },
    /// Profile attached to device
    ProfileAttachedToDevice {
        profile: ProfileNameVO,
        serial: DeviceSerialVO,
        timestamp_us: u64,
    },
    /// Profile detached from device
    ProfileDetachedFromDevice {
        profile: ProfileNameVO,
        serial: DeviceSerialVO,
        timestamp_us: u64,
    },
    /// WebSocket client connected
    WebSocketClientConnected {
        client_id: std::string::String,
        timestamp_us: u64,
    },
    /// WebSocket client disconnected
    WebSocketClientDisconnected {
        client_id: std::string::String,
        timestamp_us: u64,
    },
    /// WebSocket client authenticated
    WebSocketClientAuthenticated {
        client_id: std::string::String,
        timestamp_us: u64,
    },
    /// Session started
    SessionStarted {
        session_id: std::string::String,
        timestamp_us: u64,
    },
    /// Session ended
    SessionEnded {
        session_id: std::string::String,
        timestamp_us: u64,
    },
}

impl DomainEvent {
    /// Gets the timestamp of this event
    pub fn timestamp(&self) -> u64 {
        match self {
            Self::DeviceConnected { timestamp_us, .. } => *timestamp_us,
            Self::DeviceDisconnected { timestamp_us, .. } => *timestamp_us,
            Self::DeviceEnabled { timestamp_us, .. } => *timestamp_us,
            Self::DeviceDisabled { timestamp_us, .. } => *timestamp_us,
            Self::ProfileActivated { timestamp_us, .. } => *timestamp_us,
            Self::ProfileDeactivated { timestamp_us, .. } => *timestamp_us,
            Self::ProfileAttachedToDevice { timestamp_us, .. } => *timestamp_us,
            Self::ProfileDetachedFromDevice { timestamp_us, .. } => *timestamp_us,
            Self::WebSocketClientConnected { timestamp_us, .. } => *timestamp_us,
            Self::WebSocketClientDisconnected { timestamp_us, .. } => *timestamp_us,
            Self::WebSocketClientAuthenticated { timestamp_us, .. } => *timestamp_us,
            Self::SessionStarted { timestamp_us, .. } => *timestamp_us,
            Self::SessionEnded { timestamp_us, .. } => *timestamp_us,
        }
    }

    /// Gets a human-readable name for this event type
    pub fn event_type_name(&self) -> &'static str {
        match self {
            Self::DeviceConnected { .. } => "DeviceConnected",
            Self::DeviceDisconnected { .. } => "DeviceDisconnected",
            Self::DeviceEnabled { .. } => "DeviceEnabled",
            Self::DeviceDisabled { .. } => "DeviceDisabled",
            Self::ProfileActivated { .. } => "ProfileActivated",
            Self::ProfileDeactivated { .. } => "ProfileDeactivated",
            Self::ProfileAttachedToDevice { .. } => "ProfileAttachedToDevice",
            Self::ProfileDetachedFromDevice { .. } => "ProfileDetachedFromDevice",
            Self::WebSocketClientConnected { .. } => "WebSocketClientConnected",
            Self::WebSocketClientDisconnected { .. } => "WebSocketClientDisconnected",
            Self::WebSocketClientAuthenticated { .. } => "WebSocketClientAuthenticated",
            Self::SessionStarted { .. } => "SessionStarted",
            Self::SessionEnded { .. } => "SessionEnded",
        }
    }
}

/// Event bus for domain events
///
/// Collects and dispatches domain events.
pub struct DomainEventBus {
    events: std::vec::Vec<DomainEvent>,
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
    pub fn drain(&mut self) -> std::vec::Vec<DomainEvent> {
        core::mem::take(&mut self.events)
    }

    /// Clears all events
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Gets the count of events
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Checks if the bus is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
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
        let serial = DeviceSerialVO::new("ABC123".into()).unwrap();
        let event = DomainEvent::DeviceConnected {
            serial: serial.clone(),
            device_name: "Keyboard".into(),
            timestamp_us: 1000,
        };
        assert_eq!(event.timestamp(), 1000);

        let event = DomainEvent::DeviceDisconnected {
            serial,
            timestamp_us: 2000,
        };
        assert_eq!(event.timestamp(), 2000);
    }

    #[test]
    fn test_domain_event_type_name() {
        let serial = DeviceSerialVO::new("ABC123".into()).unwrap();
        let event = DomainEvent::DeviceConnected {
            serial: serial.clone(),
            device_name: "Keyboard".into(),
            timestamp_us: 1000,
        };
        assert_eq!(event.event_type_name(), "DeviceConnected");

        let profile = ProfileNameVO::new("Gaming".into()).unwrap();
        let event = DomainEvent::ProfileActivated {
            profile,
            timestamp_us: 2000,
        };
        assert_eq!(event.event_type_name(), "ProfileActivated");
    }

    #[test]
    fn test_event_bus_publish_and_drain() {
        let mut bus = DomainEventBus::new();

        let serial = DeviceSerialVO::new("ABC123".into()).unwrap();
        bus.publish(DomainEvent::DeviceConnected {
            serial: serial.clone(),
            device_name: "Keyboard".into(),
            timestamp_us: 1000,
        });
        bus.publish(DomainEvent::DeviceDisconnected {
            serial,
            timestamp_us: 2000,
        });

        assert_eq!(bus.len(), 2);
        assert!(!bus.is_empty());

        let events = bus.drain();
        assert_eq!(events.len(), 2);
        assert_eq!(bus.len(), 0);
        assert!(bus.is_empty());
    }

    #[test]
    fn test_event_bus_clear() {
        let mut bus = DomainEventBus::new();

        let serial = DeviceSerialVO::new("ABC123".into()).unwrap();
        bus.publish(DomainEvent::DeviceConnected {
            serial,
            device_name: "Keyboard".into(),
            timestamp_us: 1000,
        });

        assert_eq!(bus.len(), 1);

        bus.clear();
        assert_eq!(bus.len(), 0);
        assert!(bus.is_empty());
    }

    #[test]
    fn test_event_bus_events() {
        let mut bus = DomainEventBus::new();

        let serial = DeviceSerialVO::new("ABC123".into()).unwrap();
        bus.publish(DomainEvent::DeviceConnected {
            serial,
            device_name: "Keyboard".into(),
            timestamp_us: 1000,
        });

        let events = bus.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type_name(), "DeviceConnected");
    }

    #[test]
    fn test_websocket_events() {
        let mut bus = DomainEventBus::new();

        bus.publish(DomainEvent::WebSocketClientConnected {
            client_id: "127.0.0.1:8080".into(),
            timestamp_us: 1000,
        });

        bus.publish(DomainEvent::WebSocketClientAuthenticated {
            client_id: "127.0.0.1:8080".into(),
            timestamp_us: 2000,
        });

        bus.publish(DomainEvent::WebSocketClientDisconnected {
            client_id: "127.0.0.1:8080".into(),
            timestamp_us: 3000,
        });

        assert_eq!(bus.len(), 3);

        let events = bus.drain();
        assert_eq!(events[0].event_type_name(), "WebSocketClientConnected");
        assert_eq!(events[1].event_type_name(), "WebSocketClientAuthenticated");
        assert_eq!(events[2].event_type_name(), "WebSocketClientDisconnected");
    }

    #[test]
    fn test_session_events() {
        let mut bus = DomainEventBus::new();

        bus.publish(DomainEvent::SessionStarted {
            session_id: "session-123".into(),
            timestamp_us: 1000,
        });

        bus.publish(DomainEvent::SessionEnded {
            session_id: "session-123".into(),
            timestamp_us: 2000,
        });

        assert_eq!(bus.len(), 2);

        let events = bus.drain();
        assert_eq!(events[0].event_type_name(), "SessionStarted");
        assert_eq!(events[1].event_type_name(), "SessionEnded");
    }

    #[test]
    fn test_profile_device_events() {
        let mut bus = DomainEventBus::new();

        let profile = ProfileNameVO::new("Gaming".into()).unwrap();
        let serial = DeviceSerialVO::new("ABC123".into()).unwrap();

        bus.publish(DomainEvent::ProfileAttachedToDevice {
            profile: profile.clone(),
            serial: serial.clone(),
            timestamp_us: 1000,
        });

        bus.publish(DomainEvent::ProfileDetachedFromDevice {
            profile,
            serial,
            timestamp_us: 2000,
        });

        assert_eq!(bus.len(), 2);

        let events = bus.drain();
        assert_eq!(events[0].event_type_name(), "ProfileAttachedToDevice");
        assert_eq!(events[1].event_type_name(), "ProfileDetachedFromDevice");
    }
}
