//! Linux-specific domain aggregates for Platform domain

#![cfg(target_os = "linux")]

use super::value_objects::{DeviceFdVO, EventCodeVO};
use crate::platform::domain::common::DevicePathVO;
use crate::platform::domain::DomainError;

/// Evdev device aggregate root
///
/// Encapsulates a Linux evdev input device with its file descriptor, path, and state.
/// This is an aggregate because it maintains invariants across the device's
/// input capture lifecycle.
pub struct EvdevDeviceAggregate {
    /// Device path (/dev/input/eventX)
    path: DevicePathVO,
    /// File descriptor
    fd: Option<DeviceFdVO>,
    /// Device name
    name: String,
    /// Whether the device is grabbed
    grabbed: bool,
    /// Whether the device is initialized
    initialized: bool,
    /// Version counter for optimistic locking
    version: u64,
}

impl EvdevDeviceAggregate {
    /// Creates a new EvdevDevice aggregate
    pub fn new(path: DevicePathVO, name: String) -> Self {
        Self {
            path,
            fd: None,
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

    /// Gets the file descriptor
    pub fn fd(&self) -> Option<DeviceFdVO> {
        self.fd
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

    /// Opens the evdev device
    pub fn open(&mut self, fd: DeviceFdVO) -> Result<(), DomainError> {
        if self.initialized {
            return Err(DomainError::ConstraintViolation(
                "Device already opened".into(),
            ));
        }

        if !fd.is_valid() {
            return Err(DomainError::InvalidDeviceHandle(
                "Cannot open with invalid file descriptor".into(),
            ));
        }

        self.fd = Some(fd);
        self.initialized = true;
        self.version += 1;
        Ok(())
    }

    /// Grabs exclusive access via EVIOCGRAB
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

    /// Releases exclusive access
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

    /// Closes the device
    pub fn close(&mut self) -> Result<(), DomainError> {
        if !self.initialized {
            return Err(DomainError::ConstraintViolation(
                "Device not initialized".into(),
            ));
        }

        if self.grabbed {
            self.grabbed = false;
        }

        self.fd = None;
        self.initialized = false;
        self.version += 1;
        Ok(())
    }

    /// Validates the device state
    pub fn validate(&self) -> Result<(), DomainError> {
        // Must have a valid path
        if self.path.as_str().is_empty() {
            return Err(DomainError::ConstraintViolation(
                "Device path cannot be empty".into(),
            ));
        }

        // If initialized, must have a file descriptor
        if self.initialized && self.fd.is_none() {
            return Err(DomainError::ConstraintViolation(
                "Initialized device must have a file descriptor".into(),
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

/// Uinput device aggregate root
///
/// Encapsulates a Linux uinput virtual output device for event injection.
/// This is an aggregate because it maintains invariants across the device's
/// output injection lifecycle.
pub struct UinputDeviceAggregate {
    /// Device name
    name: String,
    /// File descriptor (/dev/uinput)
    fd: Option<DeviceFdVO>,
    /// Whether the device is initialized
    initialized: bool,
    /// Whether the device is created (UI_DEV_CREATE)
    created: bool,
    /// Version counter for optimistic locking
    version: u64,
}

impl UinputDeviceAggregate {
    /// Creates a new UinputDevice aggregate
    pub fn new(name: String) -> Self {
        Self {
            name,
            fd: None,
            initialized: false,
            created: false,
            version: 0,
        }
    }

    /// Gets the device name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the file descriptor
    pub fn fd(&self) -> Option<DeviceFdVO> {
        self.fd
    }

    /// Checks if the device is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Checks if the device is created
    pub fn is_created(&self) -> bool {
        self.created
    }

    /// Gets the version for optimistic locking
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Opens the uinput device
    pub fn open(&mut self, fd: DeviceFdVO) -> Result<(), DomainError> {
        if self.initialized {
            return Err(DomainError::ConstraintViolation(
                "Device already opened".into(),
            ));
        }

        if !fd.is_valid() {
            return Err(DomainError::InvalidDeviceHandle(
                "Cannot open with invalid file descriptor".into(),
            ));
        }

        self.fd = Some(fd);
        self.initialized = true;
        self.version += 1;
        Ok(())
    }

    /// Creates the uinput device (UI_DEV_CREATE)
    pub fn create(&mut self) -> Result<(), DomainError> {
        if !self.initialized {
            return Err(DomainError::ConstraintViolation(
                "Device not initialized".into(),
            ));
        }

        if self.created {
            return Err(DomainError::ConstraintViolation(
                "Device already created".into(),
            ));
        }

        self.created = true;
        self.version += 1;
        Ok(())
    }

    /// Destroys the uinput device (UI_DEV_DESTROY)
    pub fn destroy(&mut self) -> Result<(), DomainError> {
        if !self.created {
            return Err(DomainError::ConstraintViolation(
                "Device not created".into(),
            ));
        }

        self.created = false;
        self.version += 1;
        Ok(())
    }

    /// Closes the device
    pub fn close(&mut self) -> Result<(), DomainError> {
        if !self.initialized {
            return Err(DomainError::ConstraintViolation(
                "Device not initialized".into(),
            ));
        }

        if self.created {
            self.created = false;
        }

        self.fd = None;
        self.initialized = false;
        self.version += 1;
        Ok(())
    }

    /// Validates the device state
    pub fn validate(&self) -> Result<(), DomainError> {
        // Must have a non-empty name
        if self.name.is_empty() {
            return Err(DomainError::ConstraintViolation(
                "Device name cannot be empty".into(),
            ));
        }

        // If initialized, must have a file descriptor
        if self.initialized && self.fd.is_none() {
            return Err(DomainError::ConstraintViolation(
                "Initialized device must have a file descriptor".into(),
            ));
        }

        // If created, must be initialized
        if self.created && !self.initialized {
            return Err(DomainError::ConstraintViolation(
                "Created device must be initialized".into(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evdev_device_aggregate_creation() {
        let path = DevicePathVO::new("/dev/input/event0".into()).unwrap();
        let device = EvdevDeviceAggregate::new(path.clone(), "Test Keyboard".into());

        assert_eq!(device.path(), &path);
        assert_eq!(device.name(), "Test Keyboard");
        assert!(!device.is_initialized());
        assert!(!device.is_grabbed());
        assert_eq!(device.version(), 0);
    }

    #[test]
    fn test_evdev_device_aggregate_open() {
        let path = DevicePathVO::new("/dev/input/event0".into()).unwrap();
        let mut device = EvdevDeviceAggregate::new(path, "Test Keyboard".into());

        let fd = DeviceFdVO::new(3).unwrap();
        device.open(fd).unwrap();

        assert!(device.is_initialized());
        assert_eq!(device.fd(), Some(fd));
        assert_eq!(device.version(), 1);
    }

    #[test]
    fn test_evdev_device_aggregate_grab_release() {
        let path = DevicePathVO::new("/dev/input/event0".into()).unwrap();
        let mut device = EvdevDeviceAggregate::new(path, "Test Keyboard".into());

        let fd = DeviceFdVO::new(3).unwrap();
        device.open(fd).unwrap();

        // Grab
        device.grab().unwrap();
        assert!(device.is_grabbed());
        assert_eq!(device.version(), 2);

        // Release
        device.release().unwrap();
        assert!(!device.is_grabbed());
        assert_eq!(device.version(), 3);
    }

    #[test]
    fn test_evdev_device_aggregate_close() {
        let path = DevicePathVO::new("/dev/input/event0".into()).unwrap();
        let mut device = EvdevDeviceAggregate::new(path, "Test Keyboard".into());

        let fd = DeviceFdVO::new(3).unwrap();
        device.open(fd).unwrap();
        device.grab().unwrap();

        // Close
        device.close().unwrap();
        assert!(!device.is_initialized());
        assert!(!device.is_grabbed());
        assert_eq!(device.fd(), None);
    }

    #[test]
    fn test_uinput_device_aggregate_creation() {
        let device = UinputDeviceAggregate::new("Virtual Keyboard".into());

        assert_eq!(device.name(), "Virtual Keyboard");
        assert!(!device.is_initialized());
        assert!(!device.is_created());
        assert_eq!(device.version(), 0);
    }

    #[test]
    fn test_uinput_device_aggregate_open_create() {
        let mut device = UinputDeviceAggregate::new("Virtual Keyboard".into());

        let fd = DeviceFdVO::new(4).unwrap();
        device.open(fd).unwrap();

        assert!(device.is_initialized());
        assert_eq!(device.fd(), Some(fd));
        assert_eq!(device.version(), 1);

        // Create
        device.create().unwrap();
        assert!(device.is_created());
        assert_eq!(device.version(), 2);
    }

    #[test]
    fn test_uinput_device_aggregate_destroy() {
        let mut device = UinputDeviceAggregate::new("Virtual Keyboard".into());

        let fd = DeviceFdVO::new(4).unwrap();
        device.open(fd).unwrap();
        device.create().unwrap();

        // Destroy
        device.destroy().unwrap();
        assert!(!device.is_created());
        assert_eq!(device.version(), 3);
    }

    #[test]
    fn test_uinput_device_aggregate_validation() {
        let mut device = UinputDeviceAggregate::new("Virtual Keyboard".into());

        // Valid uninitialized state
        assert!(device.validate().is_ok());

        // Open
        let fd = DeviceFdVO::new(4).unwrap();
        device.open(fd).unwrap();
        assert!(device.validate().is_ok());

        // Create
        device.create().unwrap();
        assert!(device.validate().is_ok());
    }
}
