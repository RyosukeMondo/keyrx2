//! Domain aggregates for Configuration domain
//!
//! Aggregates are clusters of domain objects that can be treated as a single unit.
//! They have a root entity and enforce consistency boundaries.

use alloc::string::String;
use alloc::vec::Vec;

use crate::config::{ConfigRoot, DeviceConfig, DeviceIdentifier, KeyMapping};

use super::ConfigDomainError;

/// ProfileConfig aggregate root
///
/// Encapsulates a complete profile configuration with all devices and validation rules.
/// This is an aggregate because it maintains invariants across the entire profile.
#[derive(Clone)]
pub struct ProfileConfigAggregate {
    /// Profile name (unique identifier)
    name: String,
    /// The actual configuration
    config: ConfigRoot,
    /// Version counter for optimistic locking
    version: u64,
    /// Whether this profile is active
    active: bool,
}

impl ProfileConfigAggregate {
    /// Creates a new ProfileConfig aggregate
    pub fn new(name: String, config: ConfigRoot) -> Self {
        Self {
            name,
            config,
            version: 0,
            active: false,
        }
    }

    /// Gets the profile name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the configuration
    pub fn config(&self) -> &ConfigRoot {
        &self.config
    }

    /// Gets a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut ConfigRoot {
        self.version += 1;
        &mut self.config
    }

    /// Gets the version for optimistic locking
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Checks if this profile is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Activates this profile
    pub fn activate(&mut self) {
        self.active = true;
        self.version += 1;
    }

    /// Deactivates this profile
    pub fn deactivate(&mut self) {
        self.active = false;
        self.version += 1;
    }

    /// Adds a device configuration
    pub fn add_device(&mut self, device: DeviceConfig) -> Result<(), ConfigDomainError> {
        // Validate device pattern is not empty
        if device.identifier.pattern.is_empty() {
            return Err(ConfigDomainError::ConstraintViolation(
                "Device pattern cannot be empty".into(),
            ));
        }

        // Check for duplicate patterns
        if self
            .config
            .devices
            .iter()
            .any(|d| d.identifier.pattern == device.identifier.pattern)
        {
            return Err(ConfigDomainError::ConstraintViolation(
                "Device pattern already exists".into(),
            ));
        }

        self.config.devices.push(device);
        self.version += 1;
        Ok(())
    }

    /// Removes a device configuration by pattern
    pub fn remove_device(&mut self, pattern: &str) -> Result<(), ConfigDomainError> {
        let initial_len = self.config.devices.len();
        self.config
            .devices
            .retain(|d| d.identifier.pattern != pattern);

        if self.config.devices.len() == initial_len {
            return Err(ConfigDomainError::DeviceNotFound(pattern.into()));
        }

        self.version += 1;
        Ok(())
    }

    /// Validates this profile against domain rules
    pub fn validate(&self) -> Result<(), ConfigDomainError> {
        // Validate name is not empty
        if self.name.is_empty() {
            return Err(ConfigDomainError::ConstraintViolation(
                "Profile name cannot be empty".into(),
            ));
        }

        // Validate version
        if self.config.version.major == 0 && self.config.version.minor == 0 {
            return Err(ConfigDomainError::ConstraintViolation(
                "Invalid version 0.0.x".into(),
            ));
        }

        // Validate devices exist
        if self.config.devices.is_empty() {
            return Err(ConfigDomainError::ConstraintViolation(
                "At least one device required".into(),
            ));
        }

        Ok(())
    }
}

/// DeviceConfig aggregate root
///
/// Encapsulates device-specific configuration and mappings.
#[derive(Clone)]
pub struct DeviceConfigAggregate {
    /// Device identifier pattern
    identifier: DeviceIdentifier,
    /// Key mappings for this device
    mappings: Vec<KeyMapping>,
    /// Version counter for optimistic locking
    version: u64,
    /// Whether this device config is enabled
    enabled: bool,
}

impl DeviceConfigAggregate {
    /// Creates a new DeviceConfig aggregate
    pub fn new(identifier: DeviceIdentifier, mappings: Vec<KeyMapping>) -> Self {
        Self {
            identifier,
            mappings,
            version: 0,
            enabled: true,
        }
    }

    /// Gets the device identifier
    pub fn identifier(&self) -> &DeviceIdentifier {
        &self.identifier
    }

    /// Gets the mappings
    pub fn mappings(&self) -> &[KeyMapping] {
        &self.mappings
    }

    /// Gets the version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Checks if this device config is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enables this device config
    pub fn enable(&mut self) {
        self.enabled = true;
        self.version += 1;
    }

    /// Disables this device config
    pub fn disable(&mut self) {
        self.enabled = false;
        self.version += 1;
    }

    /// Adds a mapping
    pub fn add_mapping(&mut self, mapping: KeyMapping) {
        self.mappings.push(mapping);
        self.version += 1;
    }

    /// Removes all mappings
    pub fn clear_mappings(&mut self) {
        self.mappings.clear();
        self.version += 1;
    }

    /// Validates this device config
    pub fn validate(&self) -> Result<(), ConfigDomainError> {
        // Validate pattern is not empty
        if self.identifier.pattern.is_empty() {
            return Err(ConfigDomainError::ConstraintViolation(
                "Device pattern cannot be empty".into(),
            ));
        }

        // Validate mappings exist
        if self.mappings.is_empty() {
            return Err(ConfigDomainError::ConstraintViolation(
                "At least one mapping required".into(),
            ));
        }

        Ok(())
    }

    /// Converts to DeviceConfig
    pub fn to_device_config(&self) -> DeviceConfig {
        DeviceConfig {
            identifier: self.identifier.clone(),
            mappings: self.mappings.clone(),
        }
    }
}

/// Layer aggregate root
///
/// Encapsulates a configuration layer (like QMK layers).
#[derive(Clone)]
pub struct LayerAggregate {
    /// Layer name
    name: String,
    /// Layer ID (0-255)
    id: u8,
    /// Mappings in this layer
    mappings: Vec<KeyMapping>,
    /// Version counter
    version: u64,
    /// Whether this layer is active
    active: bool,
}

impl LayerAggregate {
    /// Creates a new Layer aggregate
    pub fn new(name: String, id: u8, mappings: Vec<KeyMapping>) -> Result<Self, ConfigDomainError> {
        // Validate name
        if name.is_empty() {
            return Err(ConfigDomainError::InvalidLayerName(
                "Layer name cannot be empty".into(),
            ));
        }

        Ok(Self {
            name,
            id,
            mappings,
            version: 0,
            active: false,
        })
    }

    /// Gets the layer name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the layer ID
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Gets the mappings
    pub fn mappings(&self) -> &[KeyMapping] {
        &self.mappings
    }

    /// Gets the version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Checks if this layer is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Activates this layer
    pub fn activate(&mut self) {
        self.active = true;
        self.version += 1;
    }

    /// Deactivates this layer
    pub fn deactivate(&mut self) {
        self.active = false;
        self.version += 1;
    }

    /// Adds a mapping to this layer
    pub fn add_mapping(&mut self, mapping: KeyMapping) {
        self.mappings.push(mapping);
        self.version += 1;
    }

    /// Clears all mappings
    pub fn clear_mappings(&mut self) {
        self.mappings.clear();
        self.version += 1;
    }

    /// Validates this layer
    pub fn validate(&self) -> Result<(), ConfigDomainError> {
        // Validate name
        if self.name.is_empty() {
            return Err(ConfigDomainError::InvalidLayerName(
                "Layer name cannot be empty".into(),
            ));
        }

        // Validate ID is in valid range (0-255 already guaranteed by u8)
        // But we can add semantic validation if needed

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{KeyCode, KeyMapping, Metadata, Version};
    use alloc::vec;

    #[test]
    fn test_profile_config_aggregate_creation() {
        let config = ConfigRoot {
            version: Version::current(),
            devices: vec![],
            metadata: Metadata {
                compilation_timestamp: 0,
                compiler_version: "test".into(),
                source_hash: "test".into(),
            },
        };

        let profile = ProfileConfigAggregate::new("test-profile".into(), config);

        assert_eq!(profile.name(), "test-profile");
        assert!(!profile.is_active());
        assert_eq!(profile.version(), 0);
    }

    #[test]
    fn test_profile_config_aggregate_activation() {
        let config = ConfigRoot {
            version: Version::current(),
            devices: vec![],
            metadata: Metadata {
                compilation_timestamp: 0,
                compiler_version: "test".into(),
                source_hash: "test".into(),
            },
        };

        let mut profile = ProfileConfigAggregate::new("test-profile".into(), config);

        profile.activate();
        assert!(profile.is_active());
        assert_eq!(profile.version(), 1);

        profile.deactivate();
        assert!(!profile.is_active());
        assert_eq!(profile.version(), 2);
    }

    #[test]
    fn test_profile_config_aggregate_add_device() {
        let config = ConfigRoot {
            version: Version::current(),
            devices: vec![],
            metadata: Metadata {
                compilation_timestamp: 0,
                compiler_version: "test".into(),
                source_hash: "test".into(),
            },
        };

        let mut profile = ProfileConfigAggregate::new("test-profile".into(), config);

        let device = DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: "*".into(),
            },
            mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
        };

        assert!(profile.add_device(device).is_ok());
        assert_eq!(profile.config().devices.len(), 1);
    }

    #[test]
    fn test_device_config_aggregate_creation() {
        let device = DeviceConfigAggregate::new(
            DeviceIdentifier {
                pattern: "*".into(),
            },
            vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
        );

        assert_eq!(device.identifier().pattern, "*");
        assert_eq!(device.mappings().len(), 1);
        assert!(device.is_enabled());
    }

    #[test]
    fn test_device_config_aggregate_validation() {
        // Valid device
        let device = DeviceConfigAggregate::new(
            DeviceIdentifier {
                pattern: "*".into(),
            },
            vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
        );
        assert!(device.validate().is_ok());

        // Invalid device (empty pattern)
        let invalid_device = DeviceConfigAggregate::new(
            DeviceIdentifier {
                pattern: "".into(),
            },
            vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
        );
        assert!(invalid_device.validate().is_err());

        // Invalid device (no mappings)
        let invalid_device2 = DeviceConfigAggregate::new(
            DeviceIdentifier {
                pattern: "*".into(),
            },
            vec![],
        );
        assert!(invalid_device2.validate().is_err());
    }

    #[test]
    fn test_layer_aggregate_creation() {
        let layer = LayerAggregate::new(
            "base".into(),
            0,
            vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
        );

        assert!(layer.is_ok());
        let layer = layer.unwrap();
        assert_eq!(layer.name(), "base");
        assert_eq!(layer.id(), 0);
        assert_eq!(layer.mappings().len(), 1);
        assert!(!layer.is_active());
    }

    #[test]
    fn test_layer_aggregate_activation() {
        let mut layer = LayerAggregate::new(
            "base".into(),
            0,
            vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
        )
        .unwrap();

        layer.activate();
        assert!(layer.is_active());
        assert_eq!(layer.version(), 1);

        layer.deactivate();
        assert!(!layer.is_active());
        assert_eq!(layer.version(), 2);
    }

    #[test]
    fn test_layer_aggregate_invalid_name() {
        let layer = LayerAggregate::new(
            "".into(),
            0,
            vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
        );

        assert!(layer.is_err());
    }
}
