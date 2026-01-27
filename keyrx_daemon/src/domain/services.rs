//! Domain services for Daemon domain
//!
//! Services contain business logic that doesn't naturally fit in entities or value objects.

use super::aggregates::{DeviceAggregate, ProfileAggregate};
use super::entities::WebSocketConnectionEntity;
use super::events::{DomainEvent, DomainEventBus};
use super::value_objects::{DeviceSerialVO, ProfileNameVO};
use super::DomainError;

/// Device identification service
///
/// Matches physical devices by their characteristics (serial, vendor ID, product ID).
pub struct DeviceIdentificationService {
    /// Event bus for publishing domain events
    event_bus: DomainEventBus,
}

impl DeviceIdentificationService {
    /// Creates a new device identification service
    pub fn new() -> Self {
        Self {
            event_bus: DomainEventBus::new(),
        }
    }

    /// Identifies a device by serial number
    pub fn identify_by_serial<'a>(
        &mut self,
        serial: &DeviceSerialVO,
        available_devices: &'a [DeviceAggregate],
    ) -> Option<&'a DeviceAggregate> {
        available_devices
            .iter()
            .find(|d| d.input_device().serial() == serial)
    }

    /// Matches a device by name pattern (regex-like)
    pub fn match_by_name_pattern<'a>(
        &mut self,
        pattern: &str,
        available_devices: &'a [DeviceAggregate],
    ) -> std::vec::Vec<&'a DeviceAggregate> {
        available_devices
            .iter()
            .filter(|d| {
                let name = d.input_device().name();
                name.contains(pattern)
            })
            .collect()
    }

    /// Validates device compatibility
    pub fn validate_device(&self, device: &DeviceAggregate) -> Result<(), DomainError> {
        // Check if device name is valid
        if device.input_device().name().is_empty() {
            return Err(DomainError::ConstraintViolation(
                "Device name cannot be empty".into(),
            ));
        }

        // Check if device serial is valid
        let serial = device.input_device().serial();
        if serial.as_str().is_empty() {
            return Err(DomainError::InvalidDeviceSerial(
                "Serial cannot be empty".into(),
            ));
        }

        Ok(())
    }

    /// Gets the event bus
    pub fn event_bus(&self) -> &DomainEventBus {
        &self.event_bus
    }

    /// Drains events from the bus
    pub fn drain_events(&mut self) -> std::vec::Vec<DomainEvent> {
        self.event_bus.drain()
    }
}

impl Default for DeviceIdentificationService {
    fn default() -> Self {
        Self::new()
    }
}

/// Profile switching service
///
/// Manages profile activation and deactivation across devices.
pub struct ProfileSwitchingService {
    /// Event bus for publishing domain events
    event_bus: DomainEventBus,
}

impl ProfileSwitchingService {
    /// Creates a new profile switching service
    pub fn new() -> Self {
        Self {
            event_bus: DomainEventBus::new(),
        }
    }

    /// Switches to a new profile
    pub fn switch_profile(
        &mut self,
        current_profile: Option<&mut ProfileAggregate>,
        new_profile: &mut ProfileAggregate,
        timestamp: u64,
    ) -> Result<(), DomainError> {
        // Deactivate current profile if exists
        if let Some(current) = current_profile {
            current.deactivate();

            self.event_bus.publish(DomainEvent::ProfileDeactivated {
                profile: current.name().clone(),
                timestamp_us: timestamp,
            });
        }

        // Activate new profile
        new_profile.activate(timestamp)?;

        self.event_bus.publish(DomainEvent::ProfileActivated {
            profile: new_profile.name().clone(),
            timestamp_us: timestamp,
        });

        Ok(())
    }

    /// Attaches a profile to a device
    pub fn attach_profile_to_device(
        &mut self,
        profile: &mut ProfileAggregate,
        device: &mut DeviceAggregate,
        timestamp: u64,
    ) -> Result<(), DomainError> {
        let serial = device.input_device().serial().clone();

        // Attach device to profile
        profile.attach_device(serial.clone())?;

        // Activate profile for device
        device.activate_profile(profile.name().clone())?;

        self.event_bus
            .publish(DomainEvent::ProfileAttachedToDevice {
                profile: profile.name().clone(),
                serial,
                timestamp_us: timestamp,
            });

        Ok(())
    }

    /// Detaches a profile from a device
    pub fn detach_profile_from_device(
        &mut self,
        profile: &mut ProfileAggregate,
        device: &mut DeviceAggregate,
        timestamp: u64,
    ) -> Result<(), DomainError> {
        let serial = device.input_device().serial().clone();

        // Detach device from profile
        profile.detach_device(&serial)?;

        // Deactivate profile for device
        device.deactivate_profile();

        self.event_bus
            .publish(DomainEvent::ProfileDetachedFromDevice {
                profile: profile.name().clone(),
                serial,
                timestamp_us: timestamp,
            });

        Ok(())
    }

    /// Validates a profile before activation
    pub fn validate_profile(&self, profile: &ProfileAggregate) -> Result<(), DomainError> {
        profile.validate()
    }

    /// Gets the event bus
    pub fn event_bus(&self) -> &DomainEventBus {
        &self.event_bus
    }

    /// Drains events from the bus
    pub fn drain_events(&mut self) -> std::vec::Vec<DomainEvent> {
        self.event_bus.drain()
    }
}

impl Default for ProfileSwitchingService {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket broadcast service
///
/// Manages WebSocket connections and broadcasts events to clients.
pub struct WebSocketBroadcastService {
    /// Active connections
    connections: std::vec::Vec<WebSocketConnectionEntity>,
    /// Event bus for publishing domain events
    event_bus: DomainEventBus,
    /// Next connection ID
    next_id: u64,
}

impl WebSocketBroadcastService {
    /// Creates a new WebSocket broadcast service
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            event_bus: DomainEventBus::new(),
            next_id: 1,
        }
    }

    /// Adds a new connection
    pub fn add_connection(
        &mut self,
        client_id: std::string::String,
        timestamp: u64,
    ) -> Result<u64, DomainError> {
        let id = self.next_id;
        self.next_id += 1;

        let connection = WebSocketConnectionEntity::new(id, client_id.clone(), timestamp);
        self.connections.push(connection);

        self.event_bus
            .publish(DomainEvent::WebSocketClientConnected {
                client_id,
                timestamp_us: timestamp,
            });

        Ok(id)
    }

    /// Removes a connection
    pub fn remove_connection(
        &mut self,
        connection_id: u64,
        timestamp: u64,
    ) -> Result<(), DomainError> {
        let pos = self
            .connections
            .iter()
            .position(|c| c.id() == connection_id)
            .ok_or_else(|| {
                DomainError::WebSocketError("Connection not found".into())
            })?;

        let connection = self.connections.remove(pos);

        self.event_bus
            .publish(DomainEvent::WebSocketClientDisconnected {
                client_id: connection.client_id().into(),
                timestamp_us: timestamp,
            });

        Ok(())
    }

    /// Authenticates a connection
    pub fn authenticate_connection(
        &mut self,
        connection_id: u64,
        timestamp: u64,
    ) -> Result<(), DomainError> {
        let connection = self
            .connections
            .iter_mut()
            .find(|c| c.id() == connection_id)
            .ok_or_else(|| {
                DomainError::WebSocketError("Connection not found".into())
            })?;

        connection.authenticate();

        self.event_bus
            .publish(DomainEvent::WebSocketClientAuthenticated {
                client_id: connection.client_id().into(),
                timestamp_us: timestamp,
            });

        Ok(())
    }

    /// Subscribes a connection to a profile
    pub fn subscribe_to_profile(
        &mut self,
        connection_id: u64,
        profile: ProfileNameVO,
    ) -> Result<(), DomainError> {
        let connection = self
            .connections
            .iter_mut()
            .find(|c| c.id() == connection_id)
            .ok_or_else(|| {
                DomainError::WebSocketError("Connection not found".into())
            })?;

        connection.subscribe_to_profile(profile);
        Ok(())
    }

    /// Broadcasts an event to all authenticated connections
    pub fn broadcast(&mut self, _message: &str, timestamp: u64) -> usize {
        let mut count = 0;

        for connection in &mut self.connections {
            if connection.is_authenticated() {
                connection.record_message(timestamp);
                count += 1;
            }
        }

        count
    }

    /// Broadcasts to connections subscribed to a specific profile
    pub fn broadcast_to_profile(
        &mut self,
        profile: &ProfileNameVO,
        _message: &str,
        timestamp: u64,
    ) -> usize {
        let mut count = 0;

        for connection in &mut self.connections {
            if let Some(subscribed) = connection.subscribed_profile() {
                if subscribed == profile && connection.is_authenticated() {
                    connection.record_message(timestamp);
                    count += 1;
                }
            }
        }

        count
    }

    /// Gets the number of active connections
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Gets the event bus
    pub fn event_bus(&self) -> &DomainEventBus {
        &self.event_bus
    }

    /// Drains events from the bus
    pub fn drain_events(&mut self) -> std::vec::Vec<DomainEvent> {
        self.event_bus.drain()
    }
}

impl Default for WebSocketBroadcastService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::InputDeviceEntity;

    #[test]
    fn test_device_identification_service() {
        let mut service = DeviceIdentificationService::new();

        let serial1 = DeviceSerialVO::new("ABC123".into()).unwrap();
        let input1 = InputDeviceEntity::new(1, serial1.clone(), "Keyboard".into(), 1000);
        let device1 = DeviceAggregate::new(input1);

        let serial2 = DeviceSerialVO::new("XYZ789".into()).unwrap();
        let input2 = InputDeviceEntity::new(2, serial2, "Mouse".into(), 1000);
        let device2 = DeviceAggregate::new(input2);

        let devices = vec![device1, device2];

        // Identify by serial
        let found = service.identify_by_serial(&serial1, &devices);
        assert!(found.is_some());
        assert_eq!(found.unwrap().input_device().name(), "Keyboard");
    }

    #[test]
    fn test_device_identification_match_by_pattern() {
        let mut service = DeviceIdentificationService::new();

        let serial1 = DeviceSerialVO::new("ABC123".into()).unwrap();
        let input1 = InputDeviceEntity::new(1, serial1, "Gaming Keyboard".into(), 1000);
        let device1 = DeviceAggregate::new(input1);

        let serial2 = DeviceSerialVO::new("XYZ789".into()).unwrap();
        let input2 = InputDeviceEntity::new(2, serial2, "Gaming Mouse".into(), 1000);
        let device2 = DeviceAggregate::new(input2);

        let devices = vec![device1, device2];

        // Match by pattern
        let matches = service.match_by_name_pattern("Gaming", &devices);
        assert_eq!(matches.len(), 2);

        let matches = service.match_by_name_pattern("Keyboard", &devices);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_profile_switching_service() {
        let mut service = ProfileSwitchingService::new();

        let name1 = ProfileNameVO::new("Default".into()).unwrap();
        let mut profile1 = ProfileAggregate::new(name1, "/path/to/default.krx".into());

        let name2 = ProfileNameVO::new("Gaming".into()).unwrap();
        let mut profile2 = ProfileAggregate::new(name2, "/path/to/gaming.krx".into());

        // Switch from no profile to profile1
        assert!(service
            .switch_profile(None, &mut profile1, 1000)
            .is_ok());
        assert!(profile1.is_active());

        // Switch from profile1 to profile2
        assert!(service
            .switch_profile(Some(&mut profile1), &mut profile2, 2000)
            .is_ok());
        assert!(!profile1.is_active());
        assert!(profile2.is_active());

        // Check events
        let events = service.drain_events();
        assert_eq!(events.len(), 3); // Activated, Deactivated, Activated
    }

    #[test]
    fn test_profile_switching_attach_detach() {
        let mut service = ProfileSwitchingService::new();

        let name = ProfileNameVO::new("Gaming".into()).unwrap();
        let mut profile = ProfileAggregate::new(name, "/path/to/gaming.krx".into());

        let serial = DeviceSerialVO::new("ABC123".into()).unwrap();
        let input = InputDeviceEntity::new(1, serial, "Keyboard".into(), 1000);
        let mut device = DeviceAggregate::new(input);

        // Attach
        assert!(service
            .attach_profile_to_device(&mut profile, &mut device, 1000)
            .is_ok());
        assert_eq!(profile.device_serials().len(), 1);
        assert!(device.active_profile().is_some());

        // Detach
        assert!(service
            .detach_profile_from_device(&mut profile, &mut device, 2000)
            .is_ok());
        assert_eq!(profile.device_serials().len(), 0);
        assert!(device.active_profile().is_none());
    }

    #[test]
    fn test_websocket_broadcast_service() {
        let mut service = WebSocketBroadcastService::new();

        // Add connection
        let conn_id = service
            .add_connection("127.0.0.1:8080".into(), 1000)
            .unwrap();
        assert_eq!(service.connection_count(), 1);

        // Authenticate
        assert!(service.authenticate_connection(conn_id, 2000).is_ok());

        // Broadcast
        let count = service.broadcast("test message", 3000);
        assert_eq!(count, 1);

        // Remove connection
        assert!(service.remove_connection(conn_id, 4000).is_ok());
        assert_eq!(service.connection_count(), 0);

        // Check events
        let events = service.drain_events();
        assert_eq!(events.len(), 3); // Connected, Authenticated, Disconnected
    }

    #[test]
    fn test_websocket_broadcast_to_profile() {
        let mut service = WebSocketBroadcastService::new();

        let profile1 = ProfileNameVO::new("Gaming".into()).unwrap();
        let profile2 = ProfileNameVO::new("Work".into()).unwrap();

        // Add two connections
        let conn1 = service
            .add_connection("127.0.0.1:8080".into(), 1000)
            .unwrap();
        let conn2 = service
            .add_connection("127.0.0.1:8081".into(), 1000)
            .unwrap();

        // Authenticate both
        service.authenticate_connection(conn1, 2000).unwrap();
        service.authenticate_connection(conn2, 2000).unwrap();

        // Subscribe to different profiles
        service
            .subscribe_to_profile(conn1, profile1.clone())
            .unwrap();
        service.subscribe_to_profile(conn2, profile2).unwrap();

        // Broadcast to profile1
        let count = service.broadcast_to_profile(&profile1, "test", 3000);
        assert_eq!(count, 1); // Only conn1 should receive
    }
}
