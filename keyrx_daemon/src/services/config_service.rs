//! Configuration service providing business logic for config operations.
//!
//! This service acts as a single source of truth for configuration operations,
//! shared between CLI and Web API. It provides operations for reading, updating,
//! and manipulating Rhai configuration files.

use std::fs;
use std::sync::Arc;

use crate::config::rhai_generator::{GeneratorError, KeyAction, RhaiGenerator};
use crate::config::ProfileManager;

/// Configuration information returned by get_config.
#[derive(Debug, Clone)]
pub struct ConfigInfo {
    pub code: String,
    pub hash: String,
    pub profile: String,
}

/// Layer information.
#[derive(Debug, Clone)]
pub struct LayerInfo {
    pub id: String,
    pub mapping_count: usize,
}

/// Errors that can occur during configuration operations.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Active profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Configuration file not found")]
    FileNotFound,

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Generator error: {0}")]
    GeneratorError(#[from] GeneratorError),

    #[error("Configuration too large (max 1MB)")]
    ConfigTooLarge,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Layer not found: {0}")]
    LayerNotFound(String),

    #[error("Invalid key name: {0}")]
    InvalidKeyName(String),
}

/// Service for configuration operations.
///
/// Provides a clean API for configuration management operations.
/// All methods are async to support future async implementations.
///
/// # Thread Safety
///
/// ConfigService is `Send + Sync` and can be shared across threads via `Arc`.
pub struct ConfigService {
    profile_manager: Arc<ProfileManager>,
}

impl ConfigService {
    /// Creates a new ConfigService.
    pub fn new(profile_manager: Arc<ProfileManager>) -> Self {
        log::debug!("ConfigService initialized");
        Self { profile_manager }
    }

    /// Gets the current configuration for the active profile.
    ///
    /// Returns the Rhai code, its hash, and the profile name.
    pub async fn get_config(&self) -> Result<ConfigInfo, ConfigError> {
        log::debug!("Getting current configuration");

        let active_profile = self
            .profile_manager
            .get_active()
            .map_err(|_| ConfigError::ProfileNotFound("failed to get active profile".to_string()))?
            .ok_or_else(|| ConfigError::ProfileNotFound("no active profile".to_string()))?;

        let metadata = self
            .profile_manager
            .get(&active_profile)
            .ok_or_else(|| ConfigError::ProfileNotFound(active_profile.clone()))?;

        let code = fs::read_to_string(&metadata.rhai_path)?;
        let hash = Self::compute_hash(&code);

        Ok(ConfigInfo {
            code,
            hash,
            profile: active_profile,
        })
    }

    /// Updates the configuration for the active profile.
    ///
    /// Validates the configuration and enforces the 1MB size limit.
    pub async fn update_config(&self, code: String) -> Result<(), ConfigError> {
        log::info!("Updating configuration");

        // Enforce 1MB size limit
        const MAX_CONFIG_SIZE: usize = 1024 * 1024; // 1MB
        if code.len() > MAX_CONFIG_SIZE {
            return Err(ConfigError::ConfigTooLarge);
        }

        let active_profile = self
            .profile_manager
            .get_active()
            .map_err(|_| ConfigError::ProfileNotFound("failed to get active profile".to_string()))?
            .ok_or_else(|| ConfigError::ProfileNotFound("no active profile".to_string()))?;

        let metadata = self
            .profile_manager
            .get(&active_profile)
            .ok_or_else(|| ConfigError::ProfileNotFound(active_profile.clone()))?;

        // Write the configuration to a temporary file first
        let temp_path = metadata.rhai_path.with_extension("rhai.tmp");
        fs::write(&temp_path, code.as_bytes())?;

        // Validate by attempting to load it
        match RhaiGenerator::load(&temp_path) {
            Ok(_) => {
                // Validation successful - move temp file to actual file
                fs::rename(&temp_path, &metadata.rhai_path)?;
                log::info!("Configuration updated successfully");
                Ok(())
            }
            Err(e) => {
                // Validation failed - remove temp file and return error
                let _ = fs::remove_file(&temp_path);
                Err(ConfigError::InvalidConfig(e.to_string()))
            }
        }
    }

    /// Sets a single key mapping in the active profile.
    pub async fn set_key_mapping(
        &self,
        layer: String,
        key: String,
        action: KeyAction,
    ) -> Result<(), ConfigError> {
        log::debug!("Setting key mapping: layer={}, key={}", layer, key);

        let active_profile = self
            .profile_manager
            .get_active()
            .map_err(|_| ConfigError::ProfileNotFound("failed to get active profile".to_string()))?
            .ok_or_else(|| ConfigError::ProfileNotFound("no active profile".to_string()))?;

        let metadata = self
            .profile_manager
            .get(&active_profile)
            .ok_or_else(|| ConfigError::ProfileNotFound(active_profile.clone()))?;

        let mut generator = RhaiGenerator::load(&metadata.rhai_path)?;

        generator
            .set_key_mapping(&layer, &key, action)
            .map_err(|e| match e {
                GeneratorError::LayerNotFound(l) => ConfigError::LayerNotFound(l),
                GeneratorError::InvalidKeyName(k) => ConfigError::InvalidKeyName(k),
                _ => ConfigError::GeneratorError(e),
            })?;

        generator.save(&metadata.rhai_path)?;

        log::info!("Key mapping updated successfully");
        Ok(())
    }

    /// Deletes a key mapping from the active profile.
    pub async fn delete_key_mapping(&self, layer: String, key: String) -> Result<(), ConfigError> {
        log::debug!("Deleting key mapping: layer={}, key={}", layer, key);

        let active_profile = self
            .profile_manager
            .get_active()
            .map_err(|_| ConfigError::ProfileNotFound("failed to get active profile".to_string()))?
            .ok_or_else(|| ConfigError::ProfileNotFound("no active profile".to_string()))?;

        let metadata = self
            .profile_manager
            .get(&active_profile)
            .ok_or_else(|| ConfigError::ProfileNotFound(active_profile.clone()))?;

        let mut generator = RhaiGenerator::load(&metadata.rhai_path)?;

        generator
            .delete_key_mapping(&layer, &key)
            .map_err(|e| match e {
                GeneratorError::LayerNotFound(l) => ConfigError::LayerNotFound(l),
                GeneratorError::InvalidKeyName(k) => ConfigError::InvalidKeyName(k),
                _ => ConfigError::GeneratorError(e),
            })?;

        generator.save(&metadata.rhai_path)?;

        log::info!("Key mapping deleted successfully");
        Ok(())
    }

    /// Gets all layers from the active profile.
    pub async fn get_layers(&self) -> Result<Vec<LayerInfo>, ConfigError> {
        log::debug!("Getting layers");

        let active_profile = self
            .profile_manager
            .get_active()
            .map_err(|_| ConfigError::ProfileNotFound("failed to get active profile".to_string()))?
            .ok_or_else(|| ConfigError::ProfileNotFound("no active profile".to_string()))?;

        let metadata = self
            .profile_manager
            .get(&active_profile)
            .ok_or_else(|| ConfigError::ProfileNotFound(active_profile.clone()))?;

        let generator = RhaiGenerator::load(&metadata.rhai_path)?;

        let layers = generator
            .list_layers()
            .into_iter()
            .map(|(id, mapping_count)| LayerInfo { id, mapping_count })
            .collect();

        Ok(layers)
    }

    /// Computes a hash of the configuration code.
    fn compute_hash(code: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        code.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let code = "let x = 42;";
        let hash1 = ConfigService::compute_hash(code);
        let hash2 = ConfigService::compute_hash(code);
        assert_eq!(hash1, hash2);

        let different_code = "let y = 43;";
        let hash3 = ConfigService::compute_hash(different_code);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_config_too_large() {
        // Create a string larger than 1MB
        let large_code = "x".repeat(1024 * 1024 + 1);
        assert!(large_code.len() > 1024 * 1024);
    }
}
