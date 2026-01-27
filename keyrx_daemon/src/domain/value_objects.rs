//! Domain value objects for Daemon domain
//!
//! Value objects are immutable and defined by their attributes, not identity.

use super::DomainError;

/// DeviceSerial value object
///
/// Represents a unique device serial number (immutable).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceSerialVO(String);

impl DeviceSerialVO {
    /// Creates a new DeviceSerial value object
    ///
    /// # Validation
    /// - Serial must not be empty
    /// - Serial must be alphanumeric with optional hyphens/underscores
    pub fn new(serial: String) -> Result<Self, DomainError> {
        if serial.is_empty() {
            return Err(DomainError::InvalidDeviceSerial(
                "Serial cannot be empty".into(),
            ));
        }

        if serial.len() > 128 {
            return Err(DomainError::InvalidDeviceSerial(
                "Serial too long (max 128 chars)".into(),
            ));
        }

        // Validate alphanumeric + hyphen/underscore
        if !serial
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(DomainError::InvalidDeviceSerial(
                "Serial must be alphanumeric with optional hyphens/underscores".into(),
            ));
        }

        Ok(Self(serial))
    }

    /// Gets the inner serial string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes self and returns the inner string
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// ProfileName value object
///
/// Represents a validated profile name (immutable).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfileNameVO(String);

impl ProfileNameVO {
    /// Creates a new ProfileName value object
    ///
    /// # Validation
    /// - Name must not be empty
    /// - Name must be 1-64 characters
    /// - Name must be alphanumeric with optional spaces/hyphens/underscores
    pub fn new(name: String) -> Result<Self, DomainError> {
        if name.is_empty() {
            return Err(DomainError::InvalidProfileName(
                "Profile name cannot be empty".into(),
            ));
        }

        if name.len() > 64 {
            return Err(DomainError::InvalidProfileName(
                "Profile name too long (max 64 chars)".into(),
            ));
        }

        // Validate alphanumeric + space/hyphen/underscore
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_')
        {
            return Err(DomainError::InvalidProfileName(
                "Profile name must be alphanumeric with optional spaces/hyphens/underscores"
                    .into(),
            ));
        }

        Ok(Self(name))
    }

    /// Gets the inner name string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes self and returns the inner string
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Port value object
///
/// Represents a validated network port number (immutable).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortVO(u16);

impl PortVO {
    /// Creates a new Port value object
    ///
    /// # Validation
    /// - Port must be in range 1024-65535 (unprivileged ports)
    pub fn new(port: u16) -> Result<Self, DomainError> {
        if port < 1024 {
            return Err(DomainError::InvalidPort(port));
        }

        Ok(Self(port))
    }

    /// Creates a Port without validation (for privileged ports)
    pub fn new_unchecked(port: u16) -> Self {
        Self(port)
    }

    /// Gets the port number
    pub fn as_u16(&self) -> u16 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_serial_vo_valid() {
        let serial = DeviceSerialVO::new("ABC123-456_789".into());
        assert!(serial.is_ok());
        assert_eq!(serial.unwrap().as_str(), "ABC123-456_789");
    }

    #[test]
    fn test_device_serial_vo_empty() {
        let serial = DeviceSerialVO::new("".into());
        assert!(serial.is_err());
    }

    #[test]
    fn test_device_serial_vo_too_long() {
        let long_serial = "A".repeat(129);
        let serial = DeviceSerialVO::new(long_serial);
        assert!(serial.is_err());
    }

    #[test]
    fn test_device_serial_vo_invalid_chars() {
        let serial = DeviceSerialVO::new("ABC@123".into());
        assert!(serial.is_err());

        let serial = DeviceSerialVO::new("ABC 123".into());
        assert!(serial.is_err());
    }

    #[test]
    fn test_device_serial_vo_equality() {
        let serial1 = DeviceSerialVO::new("ABC123".into()).unwrap();
        let serial2 = DeviceSerialVO::new("ABC123".into()).unwrap();
        let serial3 = DeviceSerialVO::new("XYZ789".into()).unwrap();

        assert_eq!(serial1, serial2);
        assert_ne!(serial1, serial3);
    }

    #[test]
    fn test_profile_name_vo_valid() {
        let name = ProfileNameVO::new("Default Profile".into());
        assert!(name.is_ok());
        assert_eq!(name.unwrap().as_str(), "Default Profile");
    }

    #[test]
    fn test_profile_name_vo_empty() {
        let name = ProfileNameVO::new("".into());
        assert!(name.is_err());
    }

    #[test]
    fn test_profile_name_vo_too_long() {
        let long_name = "A".repeat(65);
        let name = ProfileNameVO::new(long_name);
        assert!(name.is_err());
    }

    #[test]
    fn test_profile_name_vo_invalid_chars() {
        let name = ProfileNameVO::new("Profile@123".into());
        assert!(name.is_err());

        let name = ProfileNameVO::new("Profile#1".into());
        assert!(name.is_err());
    }

    #[test]
    fn test_profile_name_vo_valid_with_special() {
        let name = ProfileNameVO::new("Gaming_Profile-1".into());
        assert!(name.is_ok());
    }

    #[test]
    fn test_port_vo_valid() {
        let port = PortVO::new(8080);
        assert!(port.is_ok());
        assert_eq!(port.unwrap().as_u16(), 8080);
    }

    #[test]
    fn test_port_vo_privileged() {
        let port = PortVO::new(80);
        assert!(port.is_err());

        let port = PortVO::new(443);
        assert!(port.is_err());
    }

    #[test]
    fn test_port_vo_unchecked() {
        let port = PortVO::new_unchecked(80);
        assert_eq!(port.as_u16(), 80);
    }

    #[test]
    fn test_port_vo_equality() {
        let port1 = PortVO::new(8080).unwrap();
        let port2 = PortVO::new(8080).unwrap();
        let port3 = PortVO::new(3000).unwrap();

        assert_eq!(port1, port2);
        assert_ne!(port1, port3);
    }
}
