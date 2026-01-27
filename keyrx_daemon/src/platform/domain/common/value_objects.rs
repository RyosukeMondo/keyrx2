//! Common domain value objects for Platform domain
//!
//! Value objects are immutable and defined by their attributes, not identity.

use crate::platform::domain::DomainError;

/// Device path value object
///
/// Represents a platform-specific device path (e.g., /dev/input/event0 on Linux).
/// Ensures the path is valid and non-empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DevicePathVO {
    path: String,
}

impl DevicePathVO {
    /// Creates a new DevicePath value object
    ///
    /// # Errors
    ///
    /// Returns `DomainError::InvalidDevicePath` if the path is empty.
    pub fn new(path: String) -> Result<Self, DomainError> {
        if path.is_empty() {
            return Err(DomainError::InvalidDevicePath(
                "Path cannot be empty".into(),
            ));
        }

        Ok(Self { path })
    }

    /// Gets the inner path string
    pub fn as_str(&self) -> &str {
        &self.path
    }

    /// Checks if this is a Linux evdev path
    #[cfg(target_os = "linux")]
    pub fn is_evdev_path(&self) -> bool {
        self.path.starts_with("/dev/input/event")
    }

    /// Checks if this is a Windows device path
    #[cfg(target_os = "windows")]
    pub fn is_windows_device_path(&self) -> bool {
        self.path.starts_with(r"\\.\") || self.path.starts_with(r"\\?\")
    }
}

impl core::fmt::Display for DevicePathVO {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.path)
    }
}

/// Device handle value object
///
/// Represents an opaque platform-specific device handle.
/// On Linux, this wraps a file descriptor. On Windows, this wraps a HANDLE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceHandleVO {
    handle: i32,
}

impl DeviceHandleVO {
    /// Creates a new DeviceHandle value object
    ///
    /// # Errors
    ///
    /// Returns `DomainError::InvalidDeviceHandle` if the handle is invalid (-1).
    pub fn new(handle: i32) -> Result<Self, DomainError> {
        if handle < 0 {
            return Err(DomainError::InvalidDeviceHandle(
                "Handle cannot be negative".into(),
            ));
        }

        Ok(Self { handle })
    }

    /// Gets the raw handle value
    pub fn as_raw(&self) -> i32 {
        self.handle
    }

    /// Checks if this handle is valid (non-negative)
    pub fn is_valid(&self) -> bool {
        self.handle >= 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_path_vo_creation() {
        let path = DevicePathVO::new("/dev/input/event0".into()).unwrap();
        assert_eq!(path.as_str(), "/dev/input/event0");
    }

    #[test]
    fn test_device_path_vo_empty_path() {
        let result = DevicePathVO::new("".into());
        assert!(matches!(
            result,
            Err(DomainError::InvalidDevicePath(_))
        ));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_device_path_vo_is_evdev_path() {
        let evdev = DevicePathVO::new("/dev/input/event0".into()).unwrap();
        assert!(evdev.is_evdev_path());

        let non_evdev = DevicePathVO::new("/dev/null".into()).unwrap();
        assert!(!non_evdev.is_evdev_path());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_device_path_vo_is_windows_device_path() {
        let windows_path = DevicePathVO::new(r"\\.\PhysicalDrive0".into()).unwrap();
        assert!(windows_path.is_windows_device_path());

        let normal_path = DevicePathVO::new(r"C:\Windows".into()).unwrap();
        assert!(!normal_path.is_windows_device_path());
    }

    #[test]
    fn test_device_handle_vo_creation() {
        let handle = DeviceHandleVO::new(42).unwrap();
        assert_eq!(handle.as_raw(), 42);
        assert!(handle.is_valid());
    }

    #[test]
    fn test_device_handle_vo_invalid() {
        let result = DeviceHandleVO::new(-1);
        assert!(matches!(
            result,
            Err(DomainError::InvalidDeviceHandle(_))
        ));
    }

    #[test]
    fn test_device_handle_vo_zero() {
        let handle = DeviceHandleVO::new(0).unwrap();
        assert_eq!(handle.as_raw(), 0);
        assert!(handle.is_valid());
    }
}
