//! Domain entities for Daemon domain
//!
//! Entities have unique identity and lifecycle.

use super::value_objects::{DeviceSerialVO, ProfileNameVO};

/// Device state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    /// Device is connected and enabled
    Active,
    /// Device is connected but disabled
    Inactive,
    /// Device is disconnected
    Disconnected,
}

/// InputDevice entity with unique identity
///
/// Represents a physical input device (keyboard, mouse) with lifecycle management.
pub struct InputDeviceEntity {
    /// Unique identifier
    id: u64,
    /// Device serial number
    serial: DeviceSerialVO,
    /// Device name
    name: std::string::String,
    /// Current state
    state: DeviceState,
    /// Timestamp when connected (microseconds)
    connected_at: u64,
    /// Timestamp when last active (microseconds)
    last_active_at: u64,
}

impl InputDeviceEntity {
    /// Creates a new InputDevice entity
    pub fn new(id: u64, serial: DeviceSerialVO, name: std::string::String, timestamp: u64) -> Self {
        Self {
            id,
            serial,
            name,
            state: DeviceState::Active,
            connected_at: timestamp,
            last_active_at: timestamp,
        }
    }

    /// Gets the entity ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Gets the device serial
    pub fn serial(&self) -> &DeviceSerialVO {
        &self.serial
    }

    /// Gets the device name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the current state
    pub fn state(&self) -> DeviceState {
        self.state
    }

    /// Checks if device is active
    pub fn is_active(&self) -> bool {
        self.state == DeviceState::Active
    }

    /// Gets connection timestamp
    pub fn connected_at(&self) -> u64 {
        self.connected_at
    }

    /// Gets last active timestamp
    pub fn last_active_at(&self) -> u64 {
        self.last_active_at
    }

    /// Enables the device
    pub fn enable(&mut self) {
        self.state = DeviceState::Active;
    }

    /// Disables the device
    pub fn disable(&mut self) {
        self.state = DeviceState::Inactive;
    }

    /// Disconnects the device
    pub fn disconnect(&mut self) {
        self.state = DeviceState::Disconnected;
    }

    /// Updates last active timestamp
    pub fn mark_active(&mut self, timestamp: u64) {
        self.last_active_at = timestamp;
    }
}

/// OutputDevice entity
///
/// Represents a virtual output device created by the daemon.
pub struct OutputDeviceEntity {
    /// Unique identifier
    id: u64,
    /// Device name
    name: std::string::String,
    /// Whether device is enabled
    enabled: bool,
    /// Timestamp when created (microseconds)
    created_at: u64,
    /// Event count processed through this device
    event_count: u64,
}

impl OutputDeviceEntity {
    /// Creates a new OutputDevice entity
    pub fn new(id: u64, name: std::string::String, timestamp: u64) -> Self {
        Self {
            id,
            name,
            enabled: true,
            created_at: timestamp,
            event_count: 0,
        }
    }

    /// Gets the entity ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Gets the device name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Checks if device is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Gets creation timestamp
    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    /// Gets event count
    pub fn event_count(&self) -> u64 {
        self.event_count
    }

    /// Enables the device
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disables the device
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Increments event count
    pub fn increment_event_count(&mut self) {
        self.event_count += 1;
    }
}

/// WebSocketConnection entity
///
/// Represents an active WebSocket connection to a client.
pub struct WebSocketConnectionEntity {
    /// Unique identifier
    id: u64,
    /// Client identifier (IP:port)
    client_id: std::string::String,
    /// Connection timestamp (microseconds)
    connected_at: u64,
    /// Last message timestamp (microseconds)
    last_message_at: u64,
    /// Number of messages sent to this client
    message_count: u64,
    /// Whether connection is authenticated
    authenticated: bool,
    /// Profile subscriptions for this connection
    subscribed_profile: Option<ProfileNameVO>,
}

impl WebSocketConnectionEntity {
    /// Creates a new WebSocketConnection entity
    pub fn new(id: u64, client_id: std::string::String, timestamp: u64) -> Self {
        Self {
            id,
            client_id,
            connected_at: timestamp,
            last_message_at: timestamp,
            message_count: 0,
            authenticated: false,
            subscribed_profile: None,
        }
    }

    /// Gets the entity ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Gets the client ID
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Gets connection timestamp
    pub fn connected_at(&self) -> u64 {
        self.connected_at
    }

    /// Gets last message timestamp
    pub fn last_message_at(&self) -> u64 {
        self.last_message_at
    }

    /// Gets message count
    pub fn message_count(&self) -> u64 {
        self.message_count
    }

    /// Checks if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }

    /// Gets subscribed profile
    pub fn subscribed_profile(&self) -> Option<&ProfileNameVO> {
        self.subscribed_profile.as_ref()
    }

    /// Marks connection as authenticated
    pub fn authenticate(&mut self) {
        self.authenticated = true;
    }

    /// Subscribes to a profile
    pub fn subscribe_to_profile(&mut self, profile: ProfileNameVO) {
        self.subscribed_profile = Some(profile);
    }

    /// Unsubscribes from profile
    pub fn unsubscribe(&mut self) {
        self.subscribed_profile = None;
    }

    /// Records a message sent
    pub fn record_message(&mut self, timestamp: u64) {
        self.message_count += 1;
        self.last_message_at = timestamp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_device_entity_creation() {
        let serial = DeviceSerialVO::new("ABC123".into()).unwrap();
        let device = InputDeviceEntity::new(1, serial, "Keyboard".into(), 1000);

        assert_eq!(device.id(), 1);
        assert_eq!(device.name(), "Keyboard");
        assert_eq!(device.state(), DeviceState::Active);
        assert!(device.is_active());
        assert_eq!(device.connected_at(), 1000);
    }

    #[test]
    fn test_input_device_entity_state_transitions() {
        let serial = DeviceSerialVO::new("ABC123".into()).unwrap();
        let mut device = InputDeviceEntity::new(1, serial, "Keyboard".into(), 1000);

        assert_eq!(device.state(), DeviceState::Active);

        device.disable();
        assert_eq!(device.state(), DeviceState::Inactive);
        assert!(!device.is_active());

        device.enable();
        assert_eq!(device.state(), DeviceState::Active);
        assert!(device.is_active());

        device.disconnect();
        assert_eq!(device.state(), DeviceState::Disconnected);
        assert!(!device.is_active());
    }

    #[test]
    fn test_input_device_entity_mark_active() {
        let serial = DeviceSerialVO::new("ABC123".into()).unwrap();
        let mut device = InputDeviceEntity::new(1, serial, "Keyboard".into(), 1000);

        assert_eq!(device.last_active_at(), 1000);

        device.mark_active(2000);
        assert_eq!(device.last_active_at(), 2000);
    }

    #[test]
    fn test_output_device_entity_creation() {
        let device = OutputDeviceEntity::new(1, "VirtualKeyboard".into(), 1000);

        assert_eq!(device.id(), 1);
        assert_eq!(device.name(), "VirtualKeyboard");
        assert!(device.is_enabled());
        assert_eq!(device.created_at(), 1000);
        assert_eq!(device.event_count(), 0);
    }

    #[test]
    fn test_output_device_entity_enable_disable() {
        let mut device = OutputDeviceEntity::new(1, "VirtualKeyboard".into(), 1000);

        assert!(device.is_enabled());

        device.disable();
        assert!(!device.is_enabled());

        device.enable();
        assert!(device.is_enabled());
    }

    #[test]
    fn test_output_device_entity_event_count() {
        let mut device = OutputDeviceEntity::new(1, "VirtualKeyboard".into(), 1000);

        assert_eq!(device.event_count(), 0);

        device.increment_event_count();
        assert_eq!(device.event_count(), 1);

        device.increment_event_count();
        assert_eq!(device.event_count(), 2);
    }

    #[test]
    fn test_websocket_connection_entity_creation() {
        let conn = WebSocketConnectionEntity::new(1, "127.0.0.1:8080".into(), 1000);

        assert_eq!(conn.id(), 1);
        assert_eq!(conn.client_id(), "127.0.0.1:8080");
        assert_eq!(conn.connected_at(), 1000);
        assert!(!conn.is_authenticated());
        assert!(conn.subscribed_profile().is_none());
        assert_eq!(conn.message_count(), 0);
    }

    #[test]
    fn test_websocket_connection_entity_authentication() {
        let mut conn = WebSocketConnectionEntity::new(1, "127.0.0.1:8080".into(), 1000);

        assert!(!conn.is_authenticated());

        conn.authenticate();
        assert!(conn.is_authenticated());
    }

    #[test]
    fn test_websocket_connection_entity_subscription() {
        let mut conn = WebSocketConnectionEntity::new(1, "127.0.0.1:8080".into(), 1000);

        assert!(conn.subscribed_profile().is_none());

        let profile = ProfileNameVO::new("Gaming".into()).unwrap();
        conn.subscribe_to_profile(profile);

        assert!(conn.subscribed_profile().is_some());
        assert_eq!(conn.subscribed_profile().unwrap().as_str(), "Gaming");

        conn.unsubscribe();
        assert!(conn.subscribed_profile().is_none());
    }

    #[test]
    fn test_websocket_connection_entity_messages() {
        let mut conn = WebSocketConnectionEntity::new(1, "127.0.0.1:8080".into(), 1000);

        assert_eq!(conn.message_count(), 0);
        assert_eq!(conn.last_message_at(), 1000);

        conn.record_message(2000);
        assert_eq!(conn.message_count(), 1);
        assert_eq!(conn.last_message_at(), 2000);

        conn.record_message(3000);
        assert_eq!(conn.message_count(), 2);
        assert_eq!(conn.last_message_at(), 3000);
    }
}
