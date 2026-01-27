//! Domain value objects for Configuration domain
//!
//! Value objects are immutable and defined by their attributes, not identity.

use alloc::string::String;

use super::ConfigDomainError;

/// LayerName value object
///
/// Represents a validated layer name with domain rules.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LayerNameVO {
    name: String,
}

impl LayerNameVO {
    /// Maximum length for a layer name
    pub const MAX_LENGTH: usize = 50;

    /// Creates a new LayerName value object with validation
    pub fn new(name: String) -> Result<Self, ConfigDomainError> {
        // Validate not empty
        if name.is_empty() {
            return Err(ConfigDomainError::InvalidLayerName(
                "Layer name cannot be empty".into(),
            ));
        }

        // Validate length
        if name.len() > Self::MAX_LENGTH {
            return Err(ConfigDomainError::InvalidLayerName(
                "Layer name too long (max 50 characters)".into(),
            ));
        }

        // Validate characters (alphanumeric, underscore, hyphen)
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(ConfigDomainError::InvalidLayerName(
                "Layer name must contain only alphanumeric, underscore, or hyphen".into(),
            ));
        }

        Ok(Self { name })
    }

    /// Gets the inner name
    pub fn as_str(&self) -> &str {
        &self.name
    }

    /// Converts to String
    pub fn into_string(self) -> String {
        self.name
    }

    /// Checks if this is the base layer
    pub fn is_base(&self) -> bool {
        self.name == "base" || self.name == "default" || self.name == "0"
    }
}

impl core::fmt::Display for LayerNameVO {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Threshold value object
///
/// Represents a validated tap-hold threshold with domain rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThresholdVO {
    milliseconds: u16,
}

impl ThresholdVO {
    /// Minimum threshold in milliseconds
    pub const MIN_MS: u16 = 10;

    /// Maximum threshold in milliseconds
    pub const MAX_MS: u16 = 5000;

    /// Default threshold (200ms)
    pub const DEFAULT_MS: u16 = 200;

    /// Creates a new Threshold value object with validation
    pub fn new(milliseconds: u16) -> Result<Self, ConfigDomainError> {
        // Validate range
        if milliseconds < Self::MIN_MS {
            return Err(ConfigDomainError::InvalidThreshold(milliseconds));
        }

        if milliseconds > Self::MAX_MS {
            return Err(ConfigDomainError::InvalidThreshold(milliseconds));
        }

        Ok(Self { milliseconds })
    }

    /// Creates a threshold with the default value
    pub fn default() -> Self {
        Self {
            milliseconds: Self::DEFAULT_MS,
        }
    }

    /// Gets the threshold in milliseconds
    pub fn milliseconds(&self) -> u16 {
        self.milliseconds
    }

    /// Gets the threshold in microseconds
    pub fn microseconds(&self) -> u64 {
        (self.milliseconds as u64) * 1000
    }

    /// Checks if this is a fast threshold (< 150ms)
    pub fn is_fast(&self) -> bool {
        self.milliseconds < 150
    }

    /// Checks if this is a slow threshold (> 300ms)
    pub fn is_slow(&self) -> bool {
        self.milliseconds > 300
    }

    /// Checks if this is the default threshold
    pub fn is_default(&self) -> bool {
        self.milliseconds == Self::DEFAULT_MS
    }
}

impl core::fmt::Display for ThresholdVO {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}ms", self.milliseconds)
    }
}

impl Default for ThresholdVO {
    fn default() -> Self {
        Self::default()
    }
}

/// ModifierId value object
///
/// Represents a validated custom modifier ID with domain rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModifierIdVO {
    id: u8,
}

impl ModifierIdVO {
    /// Maximum valid modifier ID (254)
    pub const MAX_ID: u8 = 254;

    /// Creates a new ModifierId value object with validation
    pub fn new(id: u8) -> Result<Self, ConfigDomainError> {
        // Validate range (0-254, as 255 is reserved)
        if id > Self::MAX_ID {
            return Err(ConfigDomainError::InvalidModifierId(id));
        }

        Ok(Self { id })
    }

    /// Gets the inner ID
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Checks if this is a reserved system modifier (0-15)
    pub fn is_system_reserved(&self) -> bool {
        self.id < 16
    }

    /// Checks if this is user-defined (16-254)
    pub fn is_user_defined(&self) -> bool {
        self.id >= 16
    }

    /// Gets the bit index in the 255-bit state
    pub fn bit_index(&self) -> usize {
        self.id as usize
    }
}

impl core::fmt::Display for ModifierIdVO {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MD_{:02X}", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_name_vo_valid() {
        let name = LayerNameVO::new("base".into());
        assert!(name.is_ok());

        let name = name.unwrap();
        assert_eq!(name.as_str(), "base");
        assert!(name.is_base());
    }

    #[test]
    fn test_layer_name_vo_empty() {
        let name = LayerNameVO::new("".into());
        assert!(name.is_err());
    }

    #[test]
    fn test_layer_name_vo_too_long() {
        let long_name = "a".repeat(51);
        let name = LayerNameVO::new(long_name);
        assert!(name.is_err());
    }

    #[test]
    fn test_layer_name_vo_invalid_chars() {
        let name = LayerNameVO::new("layer name".into());
        assert!(name.is_err());

        let name = LayerNameVO::new("layer@name".into());
        assert!(name.is_err());
    }

    #[test]
    fn test_layer_name_vo_valid_chars() {
        let name = LayerNameVO::new("layer_1".into());
        assert!(name.is_ok());

        let name = LayerNameVO::new("layer-1".into());
        assert!(name.is_ok());

        let name = LayerNameVO::new("Layer123".into());
        assert!(name.is_ok());
    }

    #[test]
    fn test_layer_name_vo_base_detection() {
        assert!(LayerNameVO::new("base".into()).unwrap().is_base());
        assert!(LayerNameVO::new("default".into()).unwrap().is_base());
        assert!(LayerNameVO::new("0".into()).unwrap().is_base());
        assert!(!LayerNameVO::new("layer1".into()).unwrap().is_base());
    }

    #[test]
    fn test_threshold_vo_valid() {
        let threshold = ThresholdVO::new(200);
        assert!(threshold.is_ok());

        let threshold = threshold.unwrap();
        assert_eq!(threshold.milliseconds(), 200);
        assert_eq!(threshold.microseconds(), 200_000);
        assert!(threshold.is_default());
    }

    #[test]
    fn test_threshold_vo_too_low() {
        let threshold = ThresholdVO::new(5);
        assert!(threshold.is_err());
    }

    #[test]
    fn test_threshold_vo_too_high() {
        let threshold = ThresholdVO::new(6000);
        assert!(threshold.is_err());
    }

    #[test]
    fn test_threshold_vo_min_max() {
        let min = ThresholdVO::new(ThresholdVO::MIN_MS);
        assert!(min.is_ok());

        let max = ThresholdVO::new(ThresholdVO::MAX_MS);
        assert!(max.is_ok());
    }

    #[test]
    fn test_threshold_vo_speed_detection() {
        let fast = ThresholdVO::new(100).unwrap();
        assert!(fast.is_fast());
        assert!(!fast.is_slow());

        let slow = ThresholdVO::new(400).unwrap();
        assert!(!slow.is_fast());
        assert!(slow.is_slow());

        let medium = ThresholdVO::new(200).unwrap();
        assert!(!medium.is_fast());
        assert!(!medium.is_slow());
    }

    #[test]
    fn test_threshold_vo_default() {
        let threshold = ThresholdVO::default();
        assert_eq!(threshold.milliseconds(), 200);
        assert!(threshold.is_default());
    }

    #[test]
    fn test_modifier_id_vo_valid() {
        let id = ModifierIdVO::new(0);
        assert!(id.is_ok());

        let id = id.unwrap();
        assert_eq!(id.id(), 0);
        assert_eq!(id.bit_index(), 0);
    }

    #[test]
    fn test_modifier_id_vo_max() {
        let id = ModifierIdVO::new(254);
        assert!(id.is_ok());

        let invalid = ModifierIdVO::new(255);
        assert!(invalid.is_err());
    }

    #[test]
    fn test_modifier_id_vo_system_vs_user() {
        let system = ModifierIdVO::new(5).unwrap();
        assert!(system.is_system_reserved());
        assert!(!system.is_user_defined());

        let user = ModifierIdVO::new(100).unwrap();
        assert!(!user.is_system_reserved());
        assert!(user.is_user_defined());

        let boundary = ModifierIdVO::new(16).unwrap();
        assert!(!boundary.is_system_reserved());
        assert!(boundary.is_user_defined());
    }

    #[test]
    fn test_modifier_id_vo_display() {
        let id = ModifierIdVO::new(15).unwrap();
        assert_eq!(alloc::format!("{}", id), "MD_0F");

        let id = ModifierIdVO::new(255 - 1).unwrap();
        assert_eq!(alloc::format!("{}", id), "MD_FE");
    }
}
