//! Domain repository traits for Daemon domain
//!
//! Repositories provide an abstraction for data access.
//! These are traits that must be implemented by infrastructure layer.

use super::aggregates::{DeviceAggregate, ProfileAggregate};
use super::value_objects::{DeviceSerialVO, ProfileNameVO};
use super::DomainError;

/// Repository for device data
///
/// Provides access to device state and lifecycle management.
pub trait DeviceRepository {
    /// Saves a device aggregate
    fn save(&mut self, device: &DeviceAggregate) -> Result<(), DomainError>;

    /// Loads a device by serial number
    fn load(&self, serial: &DeviceSerialVO) -> Result<DeviceAggregate, DomainError>;

    /// Lists all devices
    fn list(&self) -> Result<std::vec::Vec<DeviceAggregate>, DomainError>;

    /// Checks if a device exists
    fn exists(&self, serial: &DeviceSerialVO) -> bool;

    /// Deletes a device
    fn delete(&mut self, serial: &DeviceSerialVO) -> Result<(), DomainError>;

    /// Lists active devices (connected and enabled)
    fn list_active(&self) -> Result<std::vec::Vec<DeviceAggregate>, DomainError>;
}

/// Repository for profile data
///
/// Provides access to profile configurations.
pub trait ProfileRepository {
    /// Saves a profile aggregate
    fn save(&mut self, profile: &ProfileAggregate) -> Result<(), DomainError>;

    /// Loads a profile by name
    fn load(&self, name: &ProfileNameVO) -> Result<ProfileAggregate, DomainError>;

    /// Lists all profiles
    fn list(&self) -> Result<std::vec::Vec<ProfileAggregate>, DomainError>;

    /// Checks if a profile exists
    fn exists(&self, name: &ProfileNameVO) -> bool;

    /// Deletes a profile
    fn delete(&mut self, name: &ProfileNameVO) -> Result<(), DomainError>;

    /// Gets the currently active profile
    fn get_active(&self) -> Option<&ProfileAggregate>;

    /// Lists profiles attached to a device
    fn list_by_device(
        &self,
        serial: &DeviceSerialVO,
    ) -> Result<std::vec::Vec<ProfileAggregate>, DomainError>;
}

/// Repository for settings data
///
/// Provides access to daemon settings and configuration.
pub trait SettingsRepository {
    /// Gets a setting value by key
    fn get(&self, key: &str) -> Option<std::string::String>;

    /// Sets a setting value
    fn set(&mut self, key: &str, value: std::string::String) -> Result<(), DomainError>;

    /// Deletes a setting
    fn delete(&mut self, key: &str) -> Result<(), DomainError>;

    /// Lists all setting keys
    fn list_keys(&self) -> Result<std::vec::Vec<std::string::String>, DomainError>;

    /// Clears all settings
    fn clear(&mut self) -> Result<(), DomainError>;

    /// Checks if a setting exists
    fn exists(&self, key: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::InputDeviceEntity;

    /// Mock implementation for testing DeviceRepository
    struct MockDeviceRepository {
        devices: std::vec::Vec<(DeviceSerialVO, DeviceAggregate)>,
    }

    impl MockDeviceRepository {
        fn new() -> Self {
            Self {
                devices: Vec::new(),
            }
        }
    }

    impl DeviceRepository for MockDeviceRepository {
        fn save(&mut self, device: &DeviceAggregate) -> Result<(), DomainError> {
            let serial = device.input_device().serial().clone();

            // Remove existing if present
            self.devices.retain(|(s, _)| s != &serial);

            // Clone and save (simplified - real impl would handle this better)
            let input = InputDeviceEntity::new(
                device.input_device().id(),
                serial.clone(),
                device.input_device().name().into(),
                device.input_device().connected_at(),
            );
            let new_device = DeviceAggregate::new(input);
            self.devices.push((serial, new_device));
            Ok(())
        }

        fn load(&self, serial: &DeviceSerialVO) -> Result<DeviceAggregate, DomainError> {
            self.devices
                .iter()
                .find(|(s, _)| s == serial)
                .map(|(_, d)| {
                    // Clone aggregate (simplified)
                    let input = InputDeviceEntity::new(
                        d.input_device().id(),
                        serial.clone(),
                        d.input_device().name().into(),
                        d.input_device().connected_at(),
                    );
                    DeviceAggregate::new(input)
                })
                .ok_or_else(|| DomainError::DeviceNotFound(serial.as_str().into()))
        }

        fn list(&self) -> Result<std::vec::Vec<DeviceAggregate>, DomainError> {
            Ok(self
                .devices
                .iter()
                .map(|(serial, d)| {
                    let input = InputDeviceEntity::new(
                        d.input_device().id(),
                        serial.clone(),
                        d.input_device().name().into(),
                        d.input_device().connected_at(),
                    );
                    DeviceAggregate::new(input)
                })
                .collect())
        }

        fn exists(&self, serial: &DeviceSerialVO) -> bool {
            self.devices.iter().any(|(s, _)| s == serial)
        }

        fn delete(&mut self, serial: &DeviceSerialVO) -> Result<(), DomainError> {
            let len = self.devices.len();
            self.devices.retain(|(s, _)| s != serial);
            if self.devices.len() < len {
                Ok(())
            } else {
                Err(DomainError::DeviceNotFound(serial.as_str().into()))
            }
        }

        fn list_active(&self) -> Result<std::vec::Vec<DeviceAggregate>, DomainError> {
            Ok(self
                .devices
                .iter()
                .filter(|(_, d)| d.input_device().is_active())
                .map(|(serial, d)| {
                    let input = InputDeviceEntity::new(
                        d.input_device().id(),
                        serial.clone(),
                        d.input_device().name().into(),
                        d.input_device().connected_at(),
                    );
                    DeviceAggregate::new(input)
                })
                .collect())
        }
    }

    /// Mock implementation for testing ProfileRepository
    struct MockProfileRepository {
        profiles: std::vec::Vec<(ProfileNameVO, ProfileAggregate)>,
    }

    impl MockProfileRepository {
        fn new() -> Self {
            Self {
                profiles: Vec::new(),
            }
        }
    }

    impl ProfileRepository for MockProfileRepository {
        fn save(&mut self, profile: &ProfileAggregate) -> Result<(), DomainError> {
            let name = profile.name().clone();

            // Remove existing if present
            self.profiles.retain(|(n, _)| n != &name);

            // Clone and save
            let new_profile = ProfileAggregate::new(name.clone(), profile.config_path().into());
            self.profiles.push((name, new_profile));
            Ok(())
        }

        fn load(&self, name: &ProfileNameVO) -> Result<ProfileAggregate, DomainError> {
            self.profiles
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, p)| ProfileAggregate::new(name.clone(), p.config_path().into()))
                .ok_or_else(|| DomainError::ProfileNotFound(name.as_str().into()))
        }

        fn list(&self) -> Result<std::vec::Vec<ProfileAggregate>, DomainError> {
            Ok(self
                .profiles
                .iter()
                .map(|(name, p)| ProfileAggregate::new(name.clone(), p.config_path().into()))
                .collect())
        }

        fn exists(&self, name: &ProfileNameVO) -> bool {
            self.profiles.iter().any(|(n, _)| n == name)
        }

        fn delete(&mut self, name: &ProfileNameVO) -> Result<(), DomainError> {
            let len = self.profiles.len();
            self.profiles.retain(|(n, _)| n != name);
            if self.profiles.len() < len {
                Ok(())
            } else {
                Err(DomainError::ProfileNotFound(name.as_str().into()))
            }
        }

        fn get_active(&self) -> Option<&ProfileAggregate> {
            self.profiles
                .iter()
                .find(|(_, p)| p.is_active())
                .map(|(_, p)| p)
        }

        fn list_by_device(
            &self,
            serial: &DeviceSerialVO,
        ) -> Result<std::vec::Vec<ProfileAggregate>, DomainError> {
            Ok(self
                .profiles
                .iter()
                .filter(|(_, p)| p.device_serials().contains(serial))
                .map(|(name, p)| ProfileAggregate::new(name.clone(), p.config_path().into()))
                .collect())
        }
    }

    /// Mock implementation for testing SettingsRepository
    struct MockSettingsRepository {
        settings: std::vec::Vec<(std::string::String, std::string::String)>,
    }

    impl MockSettingsRepository {
        fn new() -> Self {
            Self {
                settings: Vec::new(),
            }
        }
    }

    impl SettingsRepository for MockSettingsRepository {
        fn get(&self, key: &str) -> Option<std::string::String> {
            self.settings
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
        }

        fn set(&mut self, key: &str, value: std::string::String) -> Result<(), DomainError> {
            // Remove existing if present
            self.settings.retain(|(k, _)| k != key);
            self.settings.push((key.into(), value));
            Ok(())
        }

        fn delete(&mut self, key: &str) -> Result<(), DomainError> {
            let len = self.settings.len();
            self.settings.retain(|(k, _)| k != key);
            if self.settings.len() < len {
                Ok(())
            } else {
                Err(DomainError::ConstraintViolation(
                    "Setting not found".into(),
                ))
            }
        }

        fn list_keys(&self) -> Result<std::vec::Vec<std::string::String>, DomainError> {
            Ok(self.settings.iter().map(|(k, _)| k.clone()).collect())
        }

        fn clear(&mut self) -> Result<(), DomainError> {
            self.settings.clear();
            Ok(())
        }

        fn exists(&self, key: &str) -> bool {
            self.settings.iter().any(|(k, _)| k == key)
        }
    }

    #[test]
    fn test_mock_device_repository() {
        let mut repo = MockDeviceRepository::new();

        let serial = DeviceSerialVO::new("ABC123".into()).unwrap();
        let input = InputDeviceEntity::new(1, serial.clone(), "Keyboard".into(), 1000);
        let device = DeviceAggregate::new(input);

        // Save
        assert!(repo.save(&device).is_ok());
        assert!(repo.exists(&serial));

        // Load
        let loaded = repo.load(&serial);
        assert!(loaded.is_ok());

        // List
        let list = repo.list().unwrap();
        assert_eq!(list.len(), 1);

        // Delete
        assert!(repo.delete(&serial).is_ok());
        assert!(!repo.exists(&serial));
    }

    #[test]
    fn test_mock_profile_repository() {
        let mut repo = MockProfileRepository::new();

        let name = ProfileNameVO::new("Gaming".into()).unwrap();
        let profile = ProfileAggregate::new(name.clone(), "/path/to/config.krx".into());

        // Save
        assert!(repo.save(&profile).is_ok());
        assert!(repo.exists(&name));

        // Load
        let loaded = repo.load(&name);
        assert!(loaded.is_ok());

        // List
        let list = repo.list().unwrap();
        assert_eq!(list.len(), 1);

        // Delete
        assert!(repo.delete(&name).is_ok());
        assert!(!repo.exists(&name));
    }

    #[test]
    fn test_mock_settings_repository() {
        let mut repo = MockSettingsRepository::new();

        // Set
        assert!(repo.set("key1", "value1".into()).is_ok());
        assert!(repo.exists("key1"));

        // Get
        assert_eq!(repo.get("key1"), Some("value1".into()));
        assert_eq!(repo.get("nonexistent"), None);

        // List
        let keys = repo.list_keys().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "key1");

        // Delete
        assert!(repo.delete("key1").is_ok());
        assert!(!repo.exists("key1"));

        // Clear
        repo.set("key2", "value2".into()).unwrap();
        repo.set("key3", "value3".into()).unwrap();
        assert!(repo.clear().is_ok());
        assert_eq!(repo.list_keys().unwrap().len(), 0);
    }
}
