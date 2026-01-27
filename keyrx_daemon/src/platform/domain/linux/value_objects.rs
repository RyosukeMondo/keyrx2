//! Linux-specific domain value objects for Platform domain

#![cfg(target_os = "linux")]

use crate::platform::domain::DomainError;

/// Event code value object
///
/// Represents a Linux evdev event code (KEY_A, KEY_ENTER, etc.).
/// Event codes are used to identify specific key events in the evdev subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventCodeVO {
    code: u16,
}

impl EventCodeVO {
    /// Creates a new EventCode value object
    pub fn new(code: u16) -> Self {
        Self { code }
    }

    /// Gets the raw event code
    pub fn as_raw(&self) -> u16 {
        self.code
    }

    /// Checks if this is a key event code (0x0000-0x02FF)
    pub fn is_key_event(&self) -> bool {
        self.code < 0x0300
    }

    /// Checks if this is a synchronization event (EV_SYN = 0)
    pub fn is_sync_event(&self) -> bool {
        self.code == 0
    }
}

impl From<u16> for EventCodeVO {
    fn from(code: u16) -> Self {
        Self::new(code)
    }
}

/// Device file descriptor value object
///
/// Represents a Linux file descriptor for an evdev or uinput device.
/// File descriptors are used to interact with device files in /dev/input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceFdVO {
    fd: i32,
}

impl DeviceFdVO {
    /// Creates a new DeviceFd value object
    ///
    /// # Errors
    ///
    /// Returns `DomainError::InvalidDeviceHandle` if the file descriptor is invalid (<0).
    pub fn new(fd: i32) -> Result<Self, DomainError> {
        if fd < 0 {
            return Err(DomainError::InvalidDeviceHandle(
                "File descriptor must be non-negative".into(),
            ));
        }

        Ok(Self { fd })
    }

    /// Gets the raw file descriptor
    pub fn as_raw_fd(&self) -> i32 {
        self.fd
    }

    /// Checks if this is a valid file descriptor
    pub fn is_valid(&self) -> bool {
        self.fd >= 0
    }

    /// Checks if this is stdin (fd = 0)
    pub fn is_stdin(&self) -> bool {
        self.fd == 0
    }

    /// Checks if this is stdout (fd = 1)
    pub fn is_stdout(&self) -> bool {
        self.fd == 1
    }

    /// Checks if this is stderr (fd = 2)
    pub fn is_stderr(&self) -> bool {
        self.fd == 2
    }
}

impl TryFrom<i32> for DeviceFdVO {
    type Error = DomainError;

    fn try_from(fd: i32) -> Result<Self, Self::Error> {
        Self::new(fd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_code_vo_creation() {
        let code = EventCodeVO::new(30); // KEY_A
        assert_eq!(code.as_raw(), 30);
        assert!(code.is_key_event());
        assert!(!code.is_sync_event());
    }

    #[test]
    fn test_event_code_vo_sync_event() {
        let sync = EventCodeVO::new(0); // EV_SYN
        assert!(sync.is_sync_event());
    }

    #[test]
    fn test_event_code_vo_from_u16() {
        let code: EventCodeVO = 30u16.into();
        assert_eq!(code.as_raw(), 30);
    }

    #[test]
    fn test_device_fd_vo_creation() {
        let fd = DeviceFdVO::new(3).unwrap();
        assert_eq!(fd.as_raw_fd(), 3);
        assert!(fd.is_valid());
    }

    #[test]
    fn test_device_fd_vo_invalid() {
        let result = DeviceFdVO::new(-1);
        assert!(matches!(
            result,
            Err(DomainError::InvalidDeviceHandle(_))
        ));
    }

    #[test]
    fn test_device_fd_vo_standard_streams() {
        let stdin = DeviceFdVO::new(0).unwrap();
        assert!(stdin.is_stdin());
        assert!(!stdin.is_stdout());

        let stdout = DeviceFdVO::new(1).unwrap();
        assert!(stdout.is_stdout());
        assert!(!stdout.is_stderr());

        let stderr = DeviceFdVO::new(2).unwrap();
        assert!(stderr.is_stderr());
        assert!(!stderr.is_stdin());
    }

    #[test]
    fn test_device_fd_vo_try_from() {
        let fd: DeviceFdVO = 3i32.try_into().unwrap();
        assert_eq!(fd.as_raw_fd(), 3);

        let result: Result<DeviceFdVO, _> = (-1i32).try_into();
        assert!(result.is_err());
    }
}
