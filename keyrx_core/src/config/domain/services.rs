//! Domain services for Configuration domain
//!
//! Services contain business logic that doesn't naturally fit in entities or value objects.

use alloc::string::String;
use alloc::vec::Vec;

use crate::config::{BaseKeyMapping, ConfigRoot, DeviceConfig, KeyMapping, Version};

use super::aggregates::ProfileConfigAggregate;
use super::ConfigDomainError;

/// Configuration merging service
///
/// Merges multiple device configurations following priority rules.
pub struct ConfigMergingService;

impl ConfigMergingService {
    /// Merges multiple configurations with priority (first = highest)
    ///
    /// Device-specific configs override wildcard configs.
    /// Later mappings for the same key override earlier ones.
    pub fn merge_configs(configs: &[ConfigRoot]) -> Result<ConfigRoot, ConfigDomainError> {
        if configs.is_empty() {
            return Err(ConfigDomainError::ConstraintViolation(
                "At least one config required for merging".into(),
            ));
        }

        // Start with first config as base
        let mut merged = configs[0].clone();

        // Merge subsequent configs
        for config in &configs[1..] {
            Self::merge_into(&mut merged, config)?;
        }

        Ok(merged)
    }

    /// Merges a config into another (target is modified)
    fn merge_into(target: &mut ConfigRoot, source: &ConfigRoot) -> Result<(), ConfigDomainError> {
        // Keep highest version
        if source.version.major > target.version.major
            || (source.version.major == target.version.major
                && source.version.minor > target.version.minor)
        {
            target.version = source.version;
        }

        // Merge devices
        for source_device in &source.devices {
            if let Some(target_device) = target
                .devices
                .iter_mut()
                .find(|d| d.identifier.pattern == source_device.identifier.pattern)
            {
                // Device exists, merge mappings
                target_device.mappings.extend(source_device.mappings.clone());
            } else {
                // New device, add it
                target.devices.push(source_device.clone());
            }
        }

        Ok(())
    }

    /// Resolves device-specific config from wildcard and specific patterns
    ///
    /// Returns a merged config where device-specific mappings override wildcards.
    pub fn resolve_device_config(
        wildcard_config: &DeviceConfig,
        device_specific: &DeviceConfig,
    ) -> DeviceConfig {
        let mut merged = wildcard_config.clone();

        // Device-specific mappings override
        merged.mappings.extend(device_specific.mappings.clone());

        // Update identifier to device-specific
        merged.identifier = device_specific.identifier.clone();

        merged
    }

    /// Deduplicates mappings (keeps last occurrence)
    pub fn deduplicate_mappings(mappings: &mut Vec<KeyMapping>) {
        let mut seen_inputs = Vec::new();
        let mut result = Vec::new();

        // Process in reverse to keep last occurrence
        for mapping in mappings.iter().rev() {
            let input = match mapping {
                KeyMapping::Base(base) => match base {
                    BaseKeyMapping::Simple { from, .. } => Some(*from),
                    BaseKeyMapping::Modifier { from, .. } => Some(*from),
                    BaseKeyMapping::Lock { from, .. } => Some(*from),
                    BaseKeyMapping::TapHold { from, .. } => Some(*from),
                    BaseKeyMapping::ModifiedOutput { from, .. } => Some(*from),
                },
                KeyMapping::Conditional { .. } => None, // Don't deduplicate conditionals
            };

            if let Some(input_key) = input {
                if !seen_inputs.contains(&input_key) {
                    seen_inputs.push(input_key);
                    result.push(mapping.clone());
                }
            } else {
                result.push(mapping.clone());
            }
        }

        result.reverse();
        *mappings = result;
    }
}

/// Configuration validation service
///
/// Provides deep validation of configuration structures.
pub struct ConfigValidationService;

impl ConfigValidationService {
    /// Validates a complete profile configuration
    pub fn validate_profile(profile: &ProfileConfigAggregate) -> Result<(), ConfigDomainError> {
        // Use aggregate's own validation
        profile.validate()?;

        // Additional deep validation
        Self::validate_config(profile.config())?;

        Ok(())
    }

    /// Validates a configuration root structure
    pub fn validate_config(config: &ConfigRoot) -> Result<(), ConfigDomainError> {
        // Validate version
        if config.version.major == 0 && config.version.minor == 0 {
            return Err(ConfigDomainError::ConstraintViolation(
                "Invalid version 0.0.x".into(),
            ));
        }

        // Validate devices exist
        if config.devices.is_empty() {
            return Err(ConfigDomainError::ConstraintViolation(
                "At least one device required".into(),
            ));
        }

        // Validate each device
        for device in &config.devices {
            Self::validate_device(device)?;
        }

        Ok(())
    }

    /// Validates a device configuration
    pub fn validate_device(device: &DeviceConfig) -> Result<(), ConfigDomainError> {
        // Validate pattern
        if device.identifier.pattern.is_empty() {
            return Err(ConfigDomainError::ConstraintViolation(
                "Device pattern cannot be empty".into(),
            ));
        }

        // Validate mappings exist
        if device.mappings.is_empty() {
            return Err(ConfigDomainError::ConstraintViolation(
                "Device must have at least one mapping".into(),
            ));
        }

        // Validate each mapping
        for mapping in &device.mappings {
            Self::validate_mapping(mapping)?;
        }

        Ok(())
    }

    /// Validates a key mapping
    fn validate_mapping(mapping: &KeyMapping) -> Result<(), ConfigDomainError> {
        match mapping {
            KeyMapping::Base(base) => Self::validate_base_mapping(base),
            KeyMapping::Conditional { mappings, .. } => {
                // Validate all conditional mappings
                for base_mapping in mappings {
                    Self::validate_base_mapping(base_mapping)?;
                }
                Ok(())
            }
        }
    }

    /// Validates a base mapping
    fn validate_base_mapping(base: &BaseKeyMapping) -> Result<(), ConfigDomainError> {
        match base {
            BaseKeyMapping::Simple { from, to } => {
                if from == to {
                    return Err(ConfigDomainError::ConstraintViolation(
                        "Simple mapping cannot map key to itself".into(),
                    ));
                }
            }
            BaseKeyMapping::TapHold { threshold_ms, .. } => {
                if *threshold_ms == 0 {
                    return Err(ConfigDomainError::ConstraintViolation(
                        "TapHold threshold must be > 0".into(),
                    ));
                }
            }
            BaseKeyMapping::Modifier { modifier_id, .. } => {
                if *modifier_id == 255 {
                    return Err(ConfigDomainError::ConstraintViolation(
                        "Modifier ID 255 is reserved".into(),
                    ));
                }
            }
            BaseKeyMapping::Lock { lock_id, .. } => {
                if *lock_id == 255 {
                    return Err(ConfigDomainError::ConstraintViolation(
                        "Lock ID 255 is reserved".into(),
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }
}

/// Configuration migration service
///
/// Handles version migration between config formats.
pub struct ConfigMigrationService;

impl ConfigMigrationService {
    /// Migrates a config to the latest version
    pub fn migrate_to_latest(config: &ConfigRoot) -> Result<ConfigRoot, ConfigDomainError> {
        let current = Version::current();

        // Already at latest version
        if config.version.major == current.major
            && config.version.minor == current.minor
            && config.version.patch == current.patch
        {
            return Ok(config.clone());
        }

        // Perform migration based on source version
        let mut migrated = config.clone();

        // Example migration path (currently just one version)
        if migrated.version.major < 1 {
            migrated = Self::migrate_to_v1(&migrated)?;
        }

        Ok(migrated)
    }

    /// Migrates from v0.x to v1.0
    fn migrate_to_v1(config: &ConfigRoot) -> Result<ConfigRoot, ConfigDomainError> {
        let mut migrated = config.clone();

        // Update version
        migrated.version = Version::current();

        // Add any necessary transformations here
        // For now, just update version
        // In future: handle breaking changes, add default values, etc.

        Ok(migrated)
    }

    /// Checks if migration is needed
    pub fn needs_migration(config: &ConfigRoot) -> bool {
        let current = Version::current();
        config.version.major != current.major
            || config.version.minor != current.minor
            || config.version.patch != current.patch
    }

    /// Gets migration path description
    pub fn get_migration_path(from: &Version, to: &Version) -> String {
        alloc::format!("{}.{}.{} -> {}.{}.{}", from.major, from.minor, from.patch, to.major, to.minor, to.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DeviceIdentifier, KeyCode, Metadata};
    use alloc::vec;

    fn create_test_config() -> ConfigRoot {
        ConfigRoot {
            version: Version::current(),
            devices: vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: "*".into(),
                },
                mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
            }],
            metadata: Metadata {
                compilation_timestamp: 0,
                compiler_version: "test".into(),
                source_hash: "test".into(),
            },
        }
    }

    #[test]
    fn test_config_merging_service() {
        let config1 = create_test_config();
        let mut config2 = create_test_config();
        config2.devices[0].identifier.pattern = "USB*".into();

        let merged = ConfigMergingService::merge_configs(&[config1, config2]).unwrap();

        assert_eq!(merged.devices.len(), 2);
    }

    #[test]
    fn test_config_merging_empty() {
        let result = ConfigMergingService::merge_configs(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_deduplicate_mappings() {
        let mut mappings = vec![
            KeyMapping::simple(KeyCode::A, KeyCode::B),
            KeyMapping::simple(KeyCode::A, KeyCode::C), // Should replace first
            KeyMapping::simple(KeyCode::B, KeyCode::D),
        ];

        ConfigMergingService::deduplicate_mappings(&mut mappings);

        assert_eq!(mappings.len(), 2);
        // Should keep last A mapping (A->C) and B mapping
        assert_eq!(
            mappings[0],
            KeyMapping::simple(KeyCode::A, KeyCode::C)
        );
        assert_eq!(
            mappings[1],
            KeyMapping::simple(KeyCode::B, KeyCode::D)
        );
    }

    #[test]
    fn test_config_validation_service() {
        let config = create_test_config();
        assert!(ConfigValidationService::validate_config(&config).is_ok());

        // Invalid config
        let invalid_config = ConfigRoot {
            version: Version {
                major: 0,
                minor: 0,
                patch: 0,
            },
            devices: vec![],
            metadata: Metadata {
                compilation_timestamp: 0,
                compiler_version: "test".into(),
                source_hash: "test".into(),
            },
        };

        assert!(ConfigValidationService::validate_config(&invalid_config).is_err());
    }

    #[test]
    fn test_validate_mapping_simple_identity() {
        let mapping = BaseKeyMapping::Simple {
            from: KeyCode::A,
            to: KeyCode::A,
        };

        assert!(ConfigValidationService::validate_base_mapping(&mapping).is_err());
    }

    #[test]
    fn test_validate_mapping_threshold() {
        let mapping = BaseKeyMapping::TapHold {
            from: KeyCode::Space,
            tap: KeyCode::Space,
            hold_modifier: 0,
            threshold_ms: 0,
        };

        assert!(ConfigValidationService::validate_base_mapping(&mapping).is_err());
    }

    #[test]
    fn test_config_migration_service() {
        let config = create_test_config();

        // Should not need migration if already at current version
        assert!(!ConfigMigrationService::needs_migration(&config));

        let migrated = ConfigMigrationService::migrate_to_latest(&config).unwrap();
        assert_eq!(migrated.version, Version::current());
    }

    #[test]
    fn test_migration_path_description() {
        let from = Version {
            major: 0,
            minor: 9,
            patch: 0,
        };
        let to = Version::current();

        let path = ConfigMigrationService::get_migration_path(&from, &to);
        assert!(path.contains("0.9.0"));
        assert!(path.contains("1.0.0"));
    }
}
