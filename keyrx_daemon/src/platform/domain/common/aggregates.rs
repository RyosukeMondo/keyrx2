//! Common domain aggregates for Platform domain
//!
//! Aggregates are clusters of domain objects that can be treated as a single unit.

use super::value_objects::{DeviceHandleVO, DevicePathVO};
use crate::platform::domain::DomainError;

/// Platform device aggregate root
///
/// Encapsulates a platform-specific device with its path, handle, and metadata.
/// This is an aggregate because it maintains invariants across the device's
/// lifecycle and state.
pub struct PlatformDeviceAggregate {
    /// Device path (e.g., /dev/input/event0)
    path: DevicePathVO,
    /// Device handle (file descriptor or HANDLE)
    handle: Option<DeviceHandleVO>,
    /// Device name
    name: String,
    /// Whether the device is currently grabbed/exclusive
    grabbed: bool,
    /// Whether the device is initialized
    initialized: bool,
    /// Version counter for optimistic locking
    version: u64,
}

impl PlatformDeviceAggregate {
    /// Creates a new PlatformDevice aggregate
    pub fn new(path: DevicePathVO, name: String) -> Self {
        Self {
            path,
            handle: None,
            name,
            grabbed: false,
            initialized: false,
            version: 0,
        }
    }

    /// Gets the device path
    pub fn path(&self) -> &DevicePathVO {
        &self.path
    }

    /// Gets the device handle
    pub fn handle(&self) -> Option<DeviceHandleVO> {
        self.handle
    }

    /// Gets the device name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Checks if the device is grabbed
    pub fn is_grabbed(&self) -> bool {
        self.grabbed
    }

    /// Checks if the device is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Gets the version for optimistic locking
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Initializes the device with a handle
    pub fn initialize(&mut self, handle: DeviceHandleVO) -> Result<(), DomainError> {
        if self.initialized {
            return Err(DomainError::ConstraintViolation(
                "Device already initialized".into(),
            ));
        }

        if !handle.is_valid() {
            return Err(DomainError::InvalidDeviceHandle(
                "Cannot initialize with invalid handle".into(),
            ));
        }

        self.handle = Some(handle);
        self.initialized = true;
        self.version += 1;
        Ok(())
    }

    /// Grabs exclusive access to the device
    pub fn grab(&mut self) -> Result<(), DomainError> {
        if !self.initialized {
            return Err(DomainError::ConstraintViolation(
                "Device not initialized".into(),
            ));
        }

        if self.grabbed {
            return Err(DomainError::ConstraintViolation(
                "Device already grabbed".into(),
            ));
        }

        self.grabbed = true;
        self.version += 1;
        Ok(())
    }

    /// Releases exclusive access to the device
    pub fn release(&mut self) -> Result<(), DomainError> {
        if !self.grabbed {
            return Err(DomainError::ConstraintViolation(
                "Device not grabbed".into(),
            ));
        }

        self.grabbed = false;
        self.version += 1;
        Ok(())
    }

    /// Shuts down the device
    pub fn shutdown(&mut self) -> Result<(), DomainError> {
        if !self.initialized {
            return Err(DomainError::ConstraintViolation(
                "Device not initialized".into(),
            ));
        }

        if self.grabbed {
            self.grabbed = false;
        }

        self.handle = None;
        self.initialized = false;
        self.version += 1;
        Ok(())
    }

    /// Validates the device state
    pub fn validate(&self) -> Result<(), DomainError> {
        // Device must have a valid path
        if self.path.as_str().is_empty() {
            return Err(DomainError::ConstraintViolation(
                "Device path cannot be empty".into(),
            ));
        }

        // If initialized, must have a handle
        if self.initialized && self.handle.is_none() {
            return Err(DomainError::ConstraintViolation(
                "Initialized device must have a handle".into(),
            ));
        }

        // If grabbed, must be initialized
        if self.grabbed && !self.initialized {
            return Err(DomainError::ConstraintViolation(
                "Grabbed device must be initialized".into(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_device_aggregate_creation() {
        let path = DevicePathVO::new("/dev/input/event0".into()).unwrap();
        let device = PlatformDeviceAggregate::new(path.clone(), "Test Device".into());

        assert_eq!(device.path(), &path);
        assert_eq!(device.name(), "Test Device");
        assert!(!device.is_initialized());
        assert!(!device.is_grabbed());
        assert_eq!(device.version(), 0);
    }

    #[test]
    fn test_platform_device_aggregate_initialize() {
        let path = DevicePathVO::new("/dev/input/event0".into()).unwrap();
        let mut device = PlatformDeviceAggregate::new(path, "Test Device".into());

        let handle = DeviceHandleVO::new(42).unwrap();
        device.initialize(handle).unwrap();

        assert!(device.is_initialized());
        assert_eq!(device.handle(), Some(handle));
        assert_eq!(device.version(), 1);
    }

    #[test]
    fn test_platform_device_aggregate_double_initialize() {
        let path = DevicePathVO::new("/dev/input/event0".into()).unwrap();
        let mut device = PlatformDeviceAggregate::new(path, "Test Device".into());

        let handle = DeviceHandleVO::new(42).unwrap();
        device.initialize(handle).unwrap();

        // Second initialization should fail
        let result = device.initialize(handle);
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));
    }

    #[test]
    fn test_platform_device_aggregate_grab_release() {
        let path = DevicePathVO::new("/dev/input/event0".into()).unwrap();
        let mut device = PlatformDeviceAggregate::new(path, "Test Device".into());

        let handle = DeviceHandleVO::new(42).unwrap();
        device.initialize(handle).unwrap();

        // Grab device
        device.grab().unwrap();
        assert!(device.is_grabbed());
        assert_eq!(device.version(), 2);

        // Release device
        device.release().unwrap();
        assert!(!device.is_grabbed());
        assert_eq!(device.version(), 3);
    }

    #[test]
    fn test_platform_device_aggregate_grab_without_init() {
        let path = DevicePathVO::new("/dev/input/event0".into()).unwrap();
        let mut device = PlatformDeviceAggregate::new(path, "Test Device".into());

        // Grab without initialization should fail
        let result = device.grab();
        assert!(matches!(
            result,
            Err(DomainError::ConstraintViolation(_))
        ));
    }

    #[test]
    fn test_platform_device_aggregate_shutdown() {
        let path = DevicePathVO::new("/dev/input/event0".into()).unwrap();
        let mut device = PlatformDeviceAggregate::new(path, "Test Device".into());

        let handle = DeviceHandleVO::new(42).unwrap();
        device.initialize(handle).unwrap();
        device.grab().unwrap();

        // Shutdown
        device.shutdown().unwrap();
        assert!(!device.is_initialized());
        assert!(!device.is_grabbed());
        assert_eq!(device.handle(), None);
    }

    #[test]
    fn test_platform_device_aggregate_validation() {
        let path = DevicePathVO::new("/dev/input/event0".into()).unwrap();
        let mut device = PlatformDeviceAggregate::new(path, "Test Device".into());

        // Valid uninitialized state
        assert!(device.validate().is_ok());

        // Initialize
        let handle = DeviceHandleVO::new(42).unwrap();
        device.initialize(handle).unwrap();
        assert!(device.validate().is_ok());

        // Grab
        device.grab().unwrap();
        assert!(device.validate().is_ok());
    }
}
