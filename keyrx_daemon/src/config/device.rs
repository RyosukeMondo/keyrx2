//! Device configuration model for persistent device settings.

use serde::{Deserialize, Serialize};

/// Device scope determines whether configuration applies globally or per-device
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Scope {
    /// Configuration applies globally to all devices
    Global,
    /// Configuration applies only to this specific device
    DeviceSpecific,
}

/// Device configuration for persistent storage
///
/// This model represents device-specific settings that persist across daemon restarts.
/// Currently used by the DeviceRegistry, but can support per-device file storage if needed.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct DeviceConfig {
    /// Device serial number (unique identifier)
    pub serial: String,

    /// Keyboard layout (e.g., "ANSI_104", "JIS_109")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,

    /// Configuration scope (global or device-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<Scope>,
}

impl DeviceConfig {
    /// Create a new device configuration with the given serial
    pub fn new(serial: impl Into<String>) -> Self {
        Self {
            serial: serial.into(),
            layout: None,
            scope: None,
        }
    }

    /// Create a device configuration with layout
    pub fn with_layout(mut self, layout: impl Into<String>) -> Self {
        self.layout = Some(layout.into());
        self
    }

    /// Create a device configuration with scope
    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scope = Some(scope);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_config_new() {
        let config = DeviceConfig::new("ABC123");
        assert_eq!(config.serial, "ABC123");
        assert_eq!(config.layout, None);
        assert_eq!(config.scope, None);
    }

    #[test]
    fn test_device_config_builder() {
        let config = DeviceConfig::new("ABC123")
            .with_layout("JIS_109")
            .with_scope(Scope::DeviceSpecific);

        assert_eq!(config.serial, "ABC123");
        assert_eq!(config.layout, Some("JIS_109".to_string()));
        assert_eq!(config.scope, Some(Scope::DeviceSpecific));
    }

    #[test]
    fn test_scope_serialization() {
        let global = Scope::Global;
        let device_specific = Scope::DeviceSpecific;

        let global_json = serde_json::to_string(&global).unwrap();
        let device_json = serde_json::to_string(&device_specific).unwrap();

        assert_eq!(global_json, "\"global\"");
        assert_eq!(device_json, "\"device-specific\"");
    }

    #[test]
    fn test_device_config_serialization() {
        let config = DeviceConfig::new("ABC123")
            .with_layout("ANSI_104")
            .with_scope(Scope::Global);

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: DeviceConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.serial, "ABC123");
        assert_eq!(deserialized.layout, Some("ANSI_104".to_string()));
        assert_eq!(deserialized.scope, Some(Scope::Global));
    }

    #[test]
    fn test_default_device_config() {
        let config = DeviceConfig::default();
        assert_eq!(config.serial, "");
        assert_eq!(config.layout, None);
        assert_eq!(config.scope, None);
    }
}
