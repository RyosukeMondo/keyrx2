//! Domain repository traits for Core domain
//!
//! Repositories provide an abstraction for data access.
//! These are traits that must be implemented by infrastructure layer.

use alloc::vec::Vec;

use crate::config::ConfigRoot;
use crate::runtime::DeviceState;

use super::DomainError;

/// Repository for configuration data
///
/// Provides access to compiled configuration (.krx files).
pub trait ConfigRepository {
    /// Loads configuration by name
    fn load(&self, name: &str) -> Result<ConfigRoot, DomainError>;

    /// Lists all available configurations
    fn list(&self) -> Result<Vec<alloc::string::String>, DomainError>;

    /// Checks if a configuration exists
    fn exists(&self, name: &str) -> bool;

    /// Gets the currently active configuration
    fn get_active(&self) -> Option<&ConfigRoot>;
}

/// Repository for state persistence
///
/// Provides access to persisted device state (for restoring across restarts).
pub trait StateRepository {
    /// Saves the current state
    fn save(&mut self, state: &DeviceState) -> Result<(), DomainError>;

    /// Loads the saved state
    fn load(&self) -> Result<DeviceState, DomainError>;

    /// Clears the saved state
    fn clear(&mut self) -> Result<(), DomainError>;

    /// Checks if state exists
    fn exists(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    /// Mock implementation for testing
    struct MockConfigRepository {
        configs: Vec<(alloc::string::String, ConfigRoot)>,
    }

    impl MockConfigRepository {
        fn new() -> Self {
            Self {
                configs: Vec::new(),
            }
        }

        fn add_config(&mut self, name: alloc::string::String, config: ConfigRoot) {
            self.configs.push((name, config));
        }
    }

    impl ConfigRepository for MockConfigRepository {
        fn load(&self, name: &str) -> Result<ConfigRoot, DomainError> {
            self.configs
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, c)| c.clone())
                .ok_or(DomainError::ConfigurationNotLoaded)
        }

        fn list(&self) -> Result<Vec<alloc::string::String>, DomainError> {
            Ok(self.configs.iter().map(|(n, _)| n.clone()).collect())
        }

        fn exists(&self, name: &str) -> bool {
            self.configs.iter().any(|(n, _)| n == name)
        }

        fn get_active(&self) -> Option<&ConfigRoot> {
            self.configs.first().map(|(_, c)| c)
        }
    }

    #[test]
    fn test_mock_config_repository() {
        use crate::config::{Metadata, Version};

        let mut repo = MockConfigRepository::new();

        let config = ConfigRoot {
            version: Version::current(),
            devices: vec![],
            metadata: Metadata {
                compilation_timestamp: 0,
                compiler_version: "test".into(),
                source_hash: "test".into(),
            },
        };

        repo.add_config("test-config".into(), config);

        assert!(repo.exists("test-config"));
        assert!(!repo.exists("nonexistent"));

        let loaded = repo.load("test-config");
        assert!(loaded.is_ok());

        let list = repo.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0], "test-config");
    }
}
