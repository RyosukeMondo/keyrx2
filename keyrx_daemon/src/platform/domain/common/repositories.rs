//! Common domain repository traits for Platform domain
//!
//! Repositories provide an abstraction for data access.

use super::aggregates::PlatformDeviceAggregate;
use super::value_objects::DevicePathVO;
use crate::platform::domain::DomainError;

/// Repository for platform device data
///
/// Provides access to platform-specific device enumeration and management.
pub trait PlatformDeviceRepository {
    /// Lists all available input devices
    fn list_devices(&self) -> Result<Vec<PlatformDeviceAggregate>, DomainError>;

    /// Finds a device by path
    fn find_by_path(&self, path: &DevicePathVO) -> Result<PlatformDeviceAggregate, DomainError>;

    /// Checks if a device exists at the given path
    fn exists(&self, path: &DevicePathVO) -> bool;

    /// Gets device metadata (name, vendor ID, product ID)
    fn get_device_info(&self, path: &DevicePathVO) -> Result<DeviceInfo, DomainError>;
}

/// Device information metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    /// Device name
    pub name: String,
    /// Vendor ID
    pub vendor_id: u16,
    /// Product ID
    pub product_id: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock implementation for testing
    struct MockPlatformDeviceRepository {
        devices: Vec<(DevicePathVO, DeviceInfo)>,
    }

    impl MockPlatformDeviceRepository {
        fn new() -> Self {
            Self {
                devices: Vec::new(),
            }
        }

        fn add_device(&mut self, path: DevicePathVO, info: DeviceInfo) {
            self.devices.push((path, info));
        }
    }

    impl PlatformDeviceRepository for MockPlatformDeviceRepository {
        fn list_devices(&self) -> Result<Vec<PlatformDeviceAggregate>, DomainError> {
            Ok(self
                .devices
                .iter()
                .map(|(path, info)| {
                    PlatformDeviceAggregate::new(path.clone(), info.name.clone())
                })
                .collect())
        }

        fn find_by_path(&self, path: &DevicePathVO) -> Result<PlatformDeviceAggregate, DomainError> {
            self.devices
                .iter()
                .find(|(p, _)| p == path)
                .map(|(p, info)| PlatformDeviceAggregate::new(p.clone(), info.name.clone()))
                .ok_or_else(|| {
                    DomainError::DeviceNotFound(format!("Device not found: {}", path.as_str()))
                })
        }

        fn exists(&self, path: &DevicePathVO) -> bool {
            self.devices.iter().any(|(p, _)| p == path)
        }

        fn get_device_info(&self, path: &DevicePathVO) -> Result<DeviceInfo, DomainError> {
            self.devices
                .iter()
                .find(|(p, _)| p == path)
                .map(|(_, info)| info.clone())
                .ok_or_else(|| {
                    DomainError::DeviceNotFound(format!("Device not found: {}", path.as_str()))
                })
        }
    }

    #[test]
    fn test_mock_platform_device_repository() {
        let mut repo = MockPlatformDeviceRepository::new();

        let path = DevicePathVO::new("/dev/input/event0".into()).unwrap();
        let info = DeviceInfo {
            name: "Test Keyboard".into(),
            vendor_id: 0x1234,
            product_id: 0x5678,
        };

        repo.add_device(path.clone(), info.clone());

        assert!(repo.exists(&path));

        let device = repo.find_by_path(&path).unwrap();
        assert_eq!(device.path(), &path);
        assert_eq!(device.name(), "Test Keyboard");

        let device_info = repo.get_device_info(&path).unwrap();
        assert_eq!(device_info.name, "Test Keyboard");
        assert_eq!(device_info.vendor_id, 0x1234);
        assert_eq!(device_info.product_id, 0x5678);

        let devices = repo.list_devices().unwrap();
        assert_eq!(devices.len(), 1);
    }

    #[test]
    fn test_mock_platform_device_repository_not_found() {
        let repo = MockPlatformDeviceRepository::new();
        let path = DevicePathVO::new("/dev/input/event99".into()).unwrap();

        assert!(!repo.exists(&path));

        let result = repo.find_by_path(&path);
        assert!(matches!(result, Err(DomainError::DeviceNotFound(_))));

        let result = repo.get_device_info(&path);
        assert!(matches!(result, Err(DomainError::DeviceNotFound(_))));
    }
}
