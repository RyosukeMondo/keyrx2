//! Domain aggregates for Daemon domain
//!
//! Aggregates are clusters of domain objects that can be treated as a single unit.
//! They have a root entity and enforce consistency boundaries.

use super::entities::{InputDeviceEntity, OutputDeviceEntity};
use super::value_objects::{DeviceSerialVO, ProfileNameVO};
use super::DomainError;

/// Device aggregate root
///
/// Encapsulates device lifecycle and state management.
/// Maintains invariants between input device, output device, and their configurations.
pub struct DeviceAggregate {
    /// Input device entity
    input_device: InputDeviceEntity,
    /// Associated output device (if any)
    output_device: Option<OutputDeviceEntity>,
    /// Active profile for this device
    active_profile: Option<ProfileNameVO>,
    /// Version counter for optimistic locking
    version: u64,
}

impl DeviceAggregate {
    /// Creates a new Device aggregate
    pub fn new(input_device: InputDeviceEntity) -> Self {
        Self {
            input_device,
            output_device: None,
            active_profile: None,
            version: 0,
        }
    }

    /// Gets the input device
    pub fn input_device(&self) -> &InputDeviceEntity {
        &self.input_device
    }

    /// Gets the output device
    pub fn output_device(&self) -> Option<&OutputDeviceEntity> {
        self.output_device.as_ref()
    }

    /// Gets the active profile
    pub fn active_profile(&self) -> Option<&ProfileNameVO> {
        self.active_profile.as_ref()
    }

    /// Gets the version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Attaches an output device to this aggregate
    pub fn attach_output_device(&mut self, output: OutputDeviceEntity) -> Result<(), DomainError> {
        if self.output_device.is_some() {
            return Err(DomainError::ConstraintViolation(
                "Output device already attached".into(),
            ));
        }

        self.output_device = Some(output);
        self.version += 1;
        Ok(())
    }

    /// Detaches the output device
    pub fn detach_output_device(&mut self) -> Result<OutputDeviceEntity, DomainError> {
        self.output_device
            .take()
            .ok_or_else(|| DomainError::ConstraintViolation("No output device attached".into()))
            .map(|device| {
                self.version += 1;
                device
            })
    }

    /// Activates a profile for this device
    pub fn activate_profile(&mut self, profile: ProfileNameVO) -> Result<(), DomainError> {
        // Check if device is active
        if !self.input_device.is_active() {
            return Err(DomainError::InvalidStateTransition {
                from: "Inactive".into(),
                to: "ProfileActive".into(),
            });
        }

        self.active_profile = Some(profile);
        self.version += 1;
        Ok(())
    }

    /// Deactivates the current profile
    pub fn deactivate_profile(&mut self) {
        self.active_profile = None;
        self.version += 1;
    }

    /// Enables the device
    pub fn enable(&mut self) {
        self.input_device.enable();
        self.version += 1;
    }

    /// Disables the device
    pub fn disable(&mut self) {
        self.input_device.disable();
        self.version += 1;
    }

    /// Disconnects the device
    pub fn disconnect(&mut self) {
        self.input_device.disconnect();
        self.active_profile = None;
        self.version += 1;
    }

    /// Marks device as active with timestamp
    pub fn mark_active(&mut self, timestamp: u64) {
        self.input_device.mark_active(timestamp);
    }

    /// Processes an event through the output device
    pub fn process_event(&mut self) -> Result<(), DomainError> {
        if let Some(ref mut output) = self.output_device {
            if !output.is_enabled() {
                return Err(DomainError::ConstraintViolation(
                    "Output device is disabled".into(),
                ));
            }
            output.increment_event_count();
            Ok(())
        } else {
            Err(DomainError::ConstraintViolation(
                "No output device attached".into(),
            ))
        }
    }
}

/// Profile aggregate root
///
/// Encapsulates profile configuration and lifecycle management.
pub struct ProfileAggregate {
    /// Profile name
    name: ProfileNameVO,
    /// Configuration path (.krx file)
    config_path: std::string::String,
    /// Whether profile is currently active
    active: bool,
    /// List of device serials using this profile
    device_serials: Vec<DeviceSerialVO>,
    /// Timestamp when activated (microseconds)
    activated_at: Option<u64>,
    /// Version counter for optimistic locking
    version: u64,
}

impl ProfileAggregate {
    /// Creates a new Profile aggregate
    pub fn new(name: ProfileNameVO, config_path: std::string::String) -> Self {
        Self {
            name,
            config_path,
            active: false,
            device_serials: Vec::new(),
            activated_at: None,
            version: 0,
        }
    }

    /// Gets the profile name
    pub fn name(&self) -> &ProfileNameVO {
        &self.name
    }

    /// Gets the config path
    pub fn config_path(&self) -> &str {
        &self.config_path
    }

    /// Checks if profile is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Gets device serials using this profile
    pub fn device_serials(&self) -> &[DeviceSerialVO] {
        &self.device_serials
    }

    /// Gets activation timestamp
    pub fn activated_at(&self) -> Option<u64> {
        self.activated_at
    }

    /// Gets version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Activates the profile
    pub fn activate(&mut self, timestamp: u64) -> Result<(), DomainError> {
        if self.active {
            return Err(DomainError::ProfileAlreadyActive(
                self.name.as_str().into(),
            ));
        }

        self.active = true;
        self.activated_at = Some(timestamp);
        self.version += 1;
        Ok(())
    }

    /// Deactivates the profile
    pub fn deactivate(&mut self) {
        self.active = false;
        self.activated_at = None;
        self.version += 1;
    }

    /// Attaches a device to this profile
    pub fn attach_device(&mut self, serial: DeviceSerialVO) -> Result<(), DomainError> {
        if self.device_serials.contains(&serial) {
            return Err(DomainError::ConstraintViolation(
                "Device already attached to profile".into(),
            ));
        }

        self.device_serials.push(serial);
        self.version += 1;
        Ok(())
    }

    /// Detaches a device from this profile
    pub fn detach_device(&mut self, serial: &DeviceSerialVO) -> Result<(), DomainError> {
        let pos = self
            .device_serials
            .iter()
            .position(|s| s == serial)
            .ok_or_else(|| {
                DomainError::ConstraintViolation("Device not attached to profile".into())
            })?;

        self.device_serials.remove(pos);
        self.version += 1;
        Ok(())
    }

    /// Validates the profile
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.config_path.is_empty() {
            return Err(DomainError::ConstraintViolation(
                "Config path cannot be empty".into(),
            ));
        }

        if !self.config_path.ends_with(".krx") {
            return Err(DomainError::ConstraintViolation(
                "Config path must end with .krx".into(),
            ));
        }

        Ok(())
    }
}

/// Session aggregate root
///
/// Encapsulates user session state and lifecycle.
pub struct SessionAggregate {
    /// Session ID
    id: std::string::String,
    /// Active profiles in this session
    active_profiles: Vec<ProfileNameVO>,
    /// Connected devices in this session
    connected_devices: Vec<DeviceSerialVO>,
    /// Session start timestamp (microseconds)
    started_at: u64,
    /// Last activity timestamp (microseconds)
    last_activity_at: u64,
    /// Version counter for optimistic locking
    version: u64,
}

impl SessionAggregate {
    /// Creates a new Session aggregate
    pub fn new(id: std::string::String, timestamp: u64) -> Self {
        Self {
            id,
            active_profiles: Vec::new(),
            connected_devices: Vec::new(),
            started_at: timestamp,
            last_activity_at: timestamp,
            version: 0,
        }
    }

    /// Gets the session ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Gets active profiles
    pub fn active_profiles(&self) -> &[ProfileNameVO] {
        &self.active_profiles
    }

    /// Gets connected devices
    pub fn connected_devices(&self) -> &[DeviceSerialVO] {
        &self.connected_devices
    }

    /// Gets start timestamp
    pub fn started_at(&self) -> u64 {
        self.started_at
    }

    /// Gets last activity timestamp
    pub fn last_activity_at(&self) -> u64 {
        self.last_activity_at
    }

    /// Gets version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Adds a profile to the session
    pub fn add_profile(&mut self, profile: ProfileNameVO) -> Result<(), DomainError> {
        if self.active_profiles.contains(&profile) {
            return Err(DomainError::ConstraintViolation(
                "Profile already in session".into(),
            ));
        }

        self.active_profiles.push(profile);
        self.version += 1;
        Ok(())
    }

    /// Removes a profile from the session
    pub fn remove_profile(&mut self, profile: &ProfileNameVO) -> Result<(), DomainError> {
        let pos = self
            .active_profiles
            .iter()
            .position(|p| p == profile)
            .ok_or_else(|| DomainError::ConstraintViolation("Profile not in session".into()))?;

        self.active_profiles.remove(pos);
        self.version += 1;
        Ok(())
    }

    /// Adds a device to the session
    pub fn add_device(&mut self, serial: DeviceSerialVO) -> Result<(), DomainError> {
        if self.connected_devices.contains(&serial) {
            return Err(DomainError::DeviceAlreadyConnected(
                serial.as_str().into(),
            ));
        }

        self.connected_devices.push(serial);
        self.version += 1;
        Ok(())
    }

    /// Removes a device from the session
    pub fn remove_device(&mut self, serial: &DeviceSerialVO) -> Result<(), DomainError> {
        let pos = self
            .connected_devices
            .iter()
            .position(|s| s == serial)
            .ok_or_else(|| DomainError::DeviceNotFound(serial.as_str().into()))?;

        self.connected_devices.remove(pos);
        self.version += 1;
        Ok(())
    }

    /// Updates last activity timestamp
    pub fn mark_activity(&mut self, timestamp: u64) {
        self.last_activity_at = timestamp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_aggregate_creation() {
        let serial = DeviceSerialVO::new("ABC123".into()).unwrap();
        let input = InputDeviceEntity::new(1, serial, "Keyboard".into(), 1000);
        let aggregate = DeviceAggregate::new(input);

        assert!(aggregate.output_device().is_none());
        assert!(aggregate.active_profile().is_none());
        assert_eq!(aggregate.version(), 0);
    }

    #[test]
    fn test_device_aggregate_attach_output() {
        let serial = DeviceSerialVO::new("ABC123".into()).unwrap();
        let input = InputDeviceEntity::new(1, serial, "Keyboard".into(), 1000);
        let mut aggregate = DeviceAggregate::new(input);

        let output = OutputDeviceEntity::new(2, "Virtual".into(), 1000);
        assert!(aggregate.attach_output_device(output).is_ok());
        assert!(aggregate.output_device().is_some());
        assert_eq!(aggregate.version(), 1);
    }

    #[test]
    fn test_device_aggregate_profile_activation() {
        let serial = DeviceSerialVO::new("ABC123".into()).unwrap();
        let input = InputDeviceEntity::new(1, serial, "Keyboard".into(), 1000);
        let mut aggregate = DeviceAggregate::new(input);

        let profile = ProfileNameVO::new("Gaming".into()).unwrap();
        assert!(aggregate.activate_profile(profile).is_ok());
        assert!(aggregate.active_profile().is_some());
        assert_eq!(aggregate.version(), 1);
    }

    #[test]
    fn test_profile_aggregate_creation() {
        let name = ProfileNameVO::new("Default".into()).unwrap();
        let aggregate = ProfileAggregate::new(name, "/path/to/config.krx".into());

        assert!(!aggregate.is_active());
        assert_eq!(aggregate.device_serials().len(), 0);
        assert_eq!(aggregate.version(), 0);
    }

    #[test]
    fn test_profile_aggregate_activation() {
        let name = ProfileNameVO::new("Default".into()).unwrap();
        let mut aggregate = ProfileAggregate::new(name, "/path/to/config.krx".into());

        assert!(aggregate.activate(1000).is_ok());
        assert!(aggregate.is_active());
        assert_eq!(aggregate.activated_at(), Some(1000));
        assert_eq!(aggregate.version(), 1);

        // Cannot activate twice
        assert!(aggregate.activate(2000).is_err());
    }

    #[test]
    fn test_profile_aggregate_device_attachment() {
        let name = ProfileNameVO::new("Default".into()).unwrap();
        let mut aggregate = ProfileAggregate::new(name, "/path/to/config.krx".into());

        let serial = DeviceSerialVO::new("ABC123".into()).unwrap();
        assert!(aggregate.attach_device(serial.clone()).is_ok());
        assert_eq!(aggregate.device_serials().len(), 1);
        assert_eq!(aggregate.version(), 1);

        // Cannot attach twice
        assert!(aggregate.attach_device(serial).is_err());
    }

    #[test]
    fn test_profile_aggregate_validation() {
        let name = ProfileNameVO::new("Default".into()).unwrap();
        let aggregate = ProfileAggregate::new(name.clone(), "/path/to/config.krx".into());
        assert!(aggregate.validate().is_ok());

        let invalid = ProfileAggregate::new(name.clone(), "".into());
        assert!(invalid.validate().is_err());

        let invalid = ProfileAggregate::new(name, "/path/to/config.txt".into());
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_session_aggregate_creation() {
        let aggregate = SessionAggregate::new("session-123".into(), 1000);

        assert_eq!(aggregate.id(), "session-123");
        assert_eq!(aggregate.active_profiles().len(), 0);
        assert_eq!(aggregate.connected_devices().len(), 0);
        assert_eq!(aggregate.started_at(), 1000);
        assert_eq!(aggregate.last_activity_at(), 1000);
    }

    #[test]
    fn test_session_aggregate_profile_management() {
        let mut aggregate = SessionAggregate::new("session-123".into(), 1000);

        let profile = ProfileNameVO::new("Gaming".into()).unwrap();
        assert!(aggregate.add_profile(profile.clone()).is_ok());
        assert_eq!(aggregate.active_profiles().len(), 1);
        assert_eq!(aggregate.version(), 1);

        // Cannot add twice
        assert!(aggregate.add_profile(profile.clone()).is_err());

        // Remove profile
        assert!(aggregate.remove_profile(&profile).is_ok());
        assert_eq!(aggregate.active_profiles().len(), 0);
        assert_eq!(aggregate.version(), 2);
    }

    #[test]
    fn test_session_aggregate_device_management() {
        let mut aggregate = SessionAggregate::new("session-123".into(), 1000);

        let serial = DeviceSerialVO::new("ABC123".into()).unwrap();
        assert!(aggregate.add_device(serial.clone()).is_ok());
        assert_eq!(aggregate.connected_devices().len(), 1);
        assert_eq!(aggregate.version(), 1);

        // Cannot add twice
        assert!(aggregate.add_device(serial.clone()).is_err());

        // Remove device
        assert!(aggregate.remove_device(&serial).is_ok());
        assert_eq!(aggregate.connected_devices().len(), 0);
        assert_eq!(aggregate.version(), 2);
    }

    #[test]
    fn test_session_aggregate_activity_tracking() {
        let mut aggregate = SessionAggregate::new("session-123".into(), 1000);

        assert_eq!(aggregate.last_activity_at(), 1000);

        aggregate.mark_activity(2000);
        assert_eq!(aggregate.last_activity_at(), 2000);

        aggregate.mark_activity(3000);
        assert_eq!(aggregate.last_activity_at(), 3000);
    }
}
