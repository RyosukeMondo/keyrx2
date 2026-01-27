//! Domain repository traits for Configuration domain
//!
//! Repositories provide an abstraction for data access.
//! These are traits that must be implemented by infrastructure layer.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::aggregates::{LayerAggregate, ProfileConfigAggregate};
use super::entities::MacroEntity;
use super::ConfigDomainError;

/// Repository for profile configuration data
///
/// Provides access to profile configurations with CRUD operations.
pub trait ProfileConfigRepository {
    /// Loads a profile configuration by name
    fn load(&self, name: &str) -> Result<ProfileConfigAggregate, ConfigDomainError>;

    /// Saves a profile configuration
    fn save(&mut self, profile: &ProfileConfigAggregate) -> Result<(), ConfigDomainError>;

    /// Deletes a profile configuration by name
    fn delete(&mut self, name: &str) -> Result<(), ConfigDomainError>;

    /// Lists all available profile names
    fn list(&self) -> Result<Vec<String>, ConfigDomainError>;

    /// Checks if a profile exists
    fn exists(&self, name: &str) -> bool;

    /// Gets the currently active profile
    fn get_active(&self) -> Result<ProfileConfigAggregate, ConfigDomainError>;

    /// Sets the active profile
    fn set_active(&mut self, name: &str) -> Result<(), ConfigDomainError>;
}

/// Repository for layer persistence
///
/// Provides access to layer configurations with CRUD operations.
pub trait LayerRepository {
    /// Loads a layer by name
    fn load(&self, name: &str) -> Result<LayerAggregate, ConfigDomainError>;

    /// Saves a layer
    fn save(&mut self, layer: &LayerAggregate) -> Result<(), ConfigDomainError>;

    /// Deletes a layer by name
    fn delete(&mut self, name: &str) -> Result<(), ConfigDomainError>;

    /// Lists all available layer names
    fn list(&self) -> Result<Vec<String>, ConfigDomainError>;

    /// Checks if a layer exists
    fn exists(&self, name: &str) -> bool;

    /// Gets all layers for a profile
    fn list_by_profile(&self, profile_name: &str) -> Result<Vec<LayerAggregate>, ConfigDomainError>;

    /// Gets the active layer
    fn get_active(&self) -> Result<LayerAggregate, ConfigDomainError>;

    /// Sets the active layer
    fn set_active(&mut self, name: &str) -> Result<(), ConfigDomainError>;
}

/// Repository for macro storage
///
/// Provides access to macro definitions with CRUD operations.
pub trait MacroRepository {
    /// Loads a macro by name
    fn load(&self, name: &str) -> Result<MacroEntity, ConfigDomainError>;

    /// Saves a macro
    fn save(&mut self, macro_entity: &MacroEntity) -> Result<(), ConfigDomainError>;

    /// Deletes a macro by name
    fn delete(&mut self, name: &str) -> Result<(), ConfigDomainError>;

    /// Lists all available macro names
    fn list(&self) -> Result<Vec<String>, ConfigDomainError>;

    /// Checks if a macro exists
    fn exists(&self, name: &str) -> bool;

    /// Gets all macros for a profile
    fn list_by_profile(&self, profile_name: &str) -> Result<Vec<MacroEntity>, ConfigDomainError>;

    /// Gets enabled macros only
    fn list_enabled(&self) -> Result<Vec<MacroEntity>, ConfigDomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ConfigRoot, KeyCode, Metadata, Version};
    use alloc::vec;

    /// Mock implementation for testing ProfileConfigRepository
    struct MockProfileConfigRepository {
        profiles: Vec<(String, ProfileConfigAggregate)>,
        active_profile: Option<String>,
    }

    impl MockProfileConfigRepository {
        fn new() -> Self {
            Self {
                profiles: Vec::new(),
                active_profile: None,
            }
        }

        fn add_profile(&mut self, profile: ProfileConfigAggregate) {
            let name = profile.name().to_string();
            self.profiles.push((name.clone(), profile));
            if self.active_profile.is_none() {
                self.active_profile = Some(name);
            }
        }
    }

    impl ProfileConfigRepository for MockProfileConfigRepository {
        fn load(&self, name: &str) -> Result<ProfileConfigAggregate, ConfigDomainError> {
            self.profiles
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, p)| (*p).clone())
                .ok_or_else(|| ConfigDomainError::ProfileNotFound(name.into()))
        }

        fn save(&mut self, profile: &ProfileConfigAggregate) -> Result<(), ConfigDomainError> {
            let name = profile.name().to_string();
            if let Some(pos) = self.profiles.iter().position(|(n, _)| n == &name) {
                self.profiles[pos] = (name, profile.clone());
            } else {
                self.profiles.push((name, profile.clone()));
            }
            Ok(())
        }

        fn delete(&mut self, name: &str) -> Result<(), ConfigDomainError> {
            let initial_len = self.profiles.len();
            self.profiles.retain(|(n, _)| n != name);

            if self.profiles.len() == initial_len {
                return Err(ConfigDomainError::ProfileNotFound(name.into()));
            }

            if self.active_profile.as_deref() == Some(name) {
                self.active_profile = self.profiles.first().map(|(n, _)| n.clone());
            }

            Ok(())
        }

        fn list(&self) -> Result<Vec<String>, ConfigDomainError> {
            Ok(self.profiles.iter().map(|(n, _)| n.clone()).collect())
        }

        fn exists(&self, name: &str) -> bool {
            self.profiles.iter().any(|(n, _)| n == name)
        }

        fn get_active(&self) -> Result<ProfileConfigAggregate, ConfigDomainError> {
            let active_name = self
                .active_profile
                .as_ref()
                .ok_or_else(|| ConfigDomainError::ProfileNotFound("No active profile".into()))?;
            self.load(active_name)
        }

        fn set_active(&mut self, name: &str) -> Result<(), ConfigDomainError> {
            if !self.exists(name) {
                return Err(ConfigDomainError::ProfileNotFound(name.into()));
            }
            self.active_profile = Some(name.to_string());
            Ok(())
        }
    }

    /// Mock implementation for testing LayerRepository
    struct MockLayerRepository {
        layers: Vec<(String, LayerAggregate)>,
        active_layer: Option<String>,
    }

    impl MockLayerRepository {
        fn new() -> Self {
            Self {
                layers: Vec::new(),
                active_layer: None,
            }
        }
    }

    impl LayerRepository for MockLayerRepository {
        fn load(&self, name: &str) -> Result<LayerAggregate, ConfigDomainError> {
            self.layers
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, l)| (*l).clone())
                .ok_or_else(|| ConfigDomainError::LayerNotFound(name.into()))
        }

        fn save(&mut self, layer: &LayerAggregate) -> Result<(), ConfigDomainError> {
            let name = layer.name().to_string();
            if let Some(pos) = self.layers.iter().position(|(n, _)| n == &name) {
                self.layers[pos] = (name, layer.clone());
            } else {
                self.layers.push((name, layer.clone()));
            }
            Ok(())
        }

        fn delete(&mut self, name: &str) -> Result<(), ConfigDomainError> {
            let initial_len = self.layers.len();
            self.layers.retain(|(n, _)| n != name);

            if self.layers.len() == initial_len {
                return Err(ConfigDomainError::LayerNotFound(name.into()));
            }

            Ok(())
        }

        fn list(&self) -> Result<Vec<String>, ConfigDomainError> {
            Ok(self.layers.iter().map(|(n, _)| n.clone()).collect())
        }

        fn exists(&self, name: &str) -> bool {
            self.layers.iter().any(|(n, _)| n == name)
        }

        fn list_by_profile(&self, _profile_name: &str) -> Result<Vec<LayerAggregate>, ConfigDomainError> {
            Ok(self.layers.iter().map(|(_, l)| l.clone()).collect())
        }

        fn get_active(&self) -> Result<LayerAggregate, ConfigDomainError> {
            let active_name = self
                .active_layer
                .as_ref()
                .ok_or_else(|| ConfigDomainError::LayerNotFound("No active layer".into()))?;
            self.load(active_name)
        }

        fn set_active(&mut self, name: &str) -> Result<(), ConfigDomainError> {
            if !self.exists(name) {
                return Err(ConfigDomainError::LayerNotFound(name.into()));
            }
            self.active_layer = Some(name.to_string());
            Ok(())
        }
    }

    #[test]
    fn test_mock_profile_config_repository() {
        let mut repo = MockProfileConfigRepository::new();

        let config = ConfigRoot {
            version: Version::current(),
            devices: vec![],
            metadata: Metadata {
                compilation_timestamp: 0,
                compiler_version: "test".into(),
                source_hash: "test".into(),
            },
        };

        let profile = ProfileConfigAggregate::new("test-profile".into(), config);
        repo.add_profile(profile);

        assert!(repo.exists("test-profile"));
        assert!(!repo.exists("nonexistent"));

        let loaded = repo.load("test-profile");
        assert!(loaded.is_ok());
        assert_eq!(loaded.unwrap().name(), "test-profile");

        let list = repo.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0], "test-profile");

        let active = repo.get_active();
        assert!(active.is_ok());
        assert_eq!(active.unwrap().name(), "test-profile");
    }

    #[test]
    fn test_mock_profile_config_repository_save() {
        let mut repo = MockProfileConfigRepository::new();

        let config = ConfigRoot {
            version: Version::current(),
            devices: vec![],
            metadata: Metadata {
                compilation_timestamp: 0,
                compiler_version: "test".into(),
                source_hash: "test".into(),
            },
        };

        let profile = ProfileConfigAggregate::new("new-profile".into(), config);
        assert!(repo.save(&profile).is_ok());
        assert!(repo.exists("new-profile"));
    }

    #[test]
    fn test_mock_profile_config_repository_delete() {
        let mut repo = MockProfileConfigRepository::new();

        let config = ConfigRoot {
            version: Version::current(),
            devices: vec![],
            metadata: Metadata {
                compilation_timestamp: 0,
                compiler_version: "test".into(),
                source_hash: "test".into(),
            },
        };

        let profile = ProfileConfigAggregate::new("test-profile".into(), config);
        repo.add_profile(profile);

        assert!(repo.exists("test-profile"));
        assert!(repo.delete("test-profile").is_ok());
        assert!(!repo.exists("test-profile"));

        // Try to delete non-existent
        assert!(repo.delete("nonexistent").is_err());
    }

    #[test]
    fn test_mock_layer_repository() {
        let mut repo = MockLayerRepository::new();

        let layer = LayerAggregate::new(
            "base".into(),
            0,
            vec![crate::config::KeyMapping::simple(KeyCode::A, KeyCode::B)],
        )
        .unwrap();

        assert!(repo.save(&layer).is_ok());
        assert!(repo.exists("base"));

        let loaded = repo.load("base");
        assert!(loaded.is_ok());
        assert_eq!(loaded.unwrap().name(), "base");

        let list = repo.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0], "base");
    }
}
