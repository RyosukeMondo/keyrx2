//! Profile management with hot-reload and thread-safe activation.
//!
//! This module provides the `ProfileManager` for creating, activating, and managing
//! Rhai configuration profiles with atomic hot-reload capabilities.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Instant, SystemTime};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Maximum number of profiles allowed
const MAX_PROFILES: usize = 100;

/// Maximum profile name length
const MAX_PROFILE_NAME_LEN: usize = 32;

/// Compilation timeout in seconds
const COMPILATION_TIMEOUT_SECS: u64 = 30;

/// Profile manager for CRUD operations and hot-reload.
pub struct ProfileManager {
    config_dir: PathBuf,
    active_profile: Arc<RwLock<Option<String>>>,
    profiles: HashMap<String, ProfileMetadata>,
    activation_lock: Arc<Mutex<()>>,
}

/// Metadata for a single profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    pub name: String,
    pub rhai_path: PathBuf,
    pub krx_path: PathBuf,
    pub modified_at: SystemTime,
    pub layer_count: usize,
}

/// Template for creating new profiles.
#[derive(Debug, Clone, Copy)]
pub enum ProfileTemplate {
    /// Empty configuration with just base layer
    Blank,
    /// QMK-style layer system example
    QmkLayers,
}

/// Result of profile activation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationResult {
    pub compile_time_ms: u64,
    pub reload_time_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

/// Errors that can occur during profile operations.
#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("Profile not found: {0}")]
    NotFound(String),

    #[error("Invalid profile name: {0}")]
    InvalidName(String),

    #[error("Compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Compilation timeout (exceeded {COMPILATION_TIMEOUT_SECS}s)")]
    CompilationTimeout,

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Profile limit exceeded (max {MAX_PROFILES})")]
    ProfileLimitExceeded,

    #[error("Disk space exhausted")]
    DiskSpaceExhausted,

    #[error("Profile already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid template")]
    InvalidTemplate,
}

impl ProfileManager {
    /// Create a new profile manager with the specified config directory.
    pub fn new(config_dir: PathBuf) -> Result<Self, ProfileError> {
        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        // Create profiles subdirectory
        let profiles_dir = config_dir.join("profiles");
        if !profiles_dir.exists() {
            fs::create_dir_all(&profiles_dir)?;
        }

        let mut manager = Self {
            config_dir,
            active_profile: Arc::new(RwLock::new(None)),
            profiles: HashMap::new(),
            activation_lock: Arc::new(Mutex::new(())),
        };

        // Scan for existing profiles
        manager.scan_profiles()?;

        Ok(manager)
    }

    /// Scan the profiles directory for .rhai files.
    pub fn scan_profiles(&mut self) -> Result<(), ProfileError> {
        let profiles_dir = self.config_dir.join("profiles");
        if !profiles_dir.exists() {
            return Ok(());
        }

        self.profiles.clear();

        for entry in fs::read_dir(&profiles_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("rhai") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    let metadata = self.load_profile_metadata(name)?;
                    self.profiles.insert(name.to_string(), metadata);
                }
            }
        }

        Ok(())
    }

    /// Load metadata for a profile by name.
    fn load_profile_metadata(&self, name: &str) -> Result<ProfileMetadata, ProfileError> {
        let rhai_path = self
            .config_dir
            .join("profiles")
            .join(format!("{}.rhai", name));
        let krx_path = self
            .config_dir
            .join("profiles")
            .join(format!("{}.krx", name));

        if !rhai_path.exists() {
            return Err(ProfileError::NotFound(name.to_string()));
        }

        let modified_at = rhai_path.metadata()?.modified()?;

        // Try to read layer count from file (simple heuristic for now)
        let layer_count = Self::count_layers(&rhai_path)?;

        Ok(ProfileMetadata {
            name: name.to_string(),
            rhai_path,
            krx_path,
            modified_at,
            layer_count,
        })
    }

    /// Count layers in a Rhai file (simple heuristic).
    fn count_layers(path: &Path) -> Result<usize, ProfileError> {
        let content = fs::read_to_string(path)?;
        let count = content.matches("layer(").count();
        Ok(count.max(1)) // At least one layer
    }

    /// Validate profile name.
    fn validate_name(name: &str) -> Result<(), ProfileError> {
        if name.is_empty() {
            return Err(ProfileError::InvalidName(
                "Name cannot be empty".to_string(),
            ));
        }

        if name.len() > MAX_PROFILE_NAME_LEN {
            return Err(ProfileError::InvalidName(format!(
                "Name too long (max {} chars)",
                MAX_PROFILE_NAME_LEN
            )));
        }

        // Allow alphanumeric, dash, underscore
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ProfileError::InvalidName(
                "Name can only contain alphanumeric characters, dashes, and underscores"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Create a new profile from a template.
    pub fn create(
        &mut self,
        name: &str,
        template: ProfileTemplate,
    ) -> Result<ProfileMetadata, ProfileError> {
        Self::validate_name(name)?;

        if self.profiles.len() >= MAX_PROFILES {
            return Err(ProfileError::ProfileLimitExceeded);
        }

        let rhai_path = self
            .config_dir
            .join("profiles")
            .join(format!("{}.rhai", name));
        if rhai_path.exists() {
            return Err(ProfileError::AlreadyExists(name.to_string()));
        }

        // Generate template content
        let content = match template {
            ProfileTemplate::Blank => Self::generate_blank_template(),
            ProfileTemplate::QmkLayers => Self::generate_qmk_template(),
        };

        fs::write(&rhai_path, content)?;

        let metadata = self.load_profile_metadata(name)?;
        self.profiles.insert(name.to_string(), metadata.clone());

        Ok(metadata)
    }

    /// Generate blank template.
    fn generate_blank_template() -> String {
        r#"// KeyRx2 Configuration
// Base layer - passthrough by default
layer("base", #{
    // Add your key mappings here
});
"#
        .to_string()
    }

    /// Generate QMK-style template.
    fn generate_qmk_template() -> String {
        r#"// KeyRx2 Configuration - QMK-style layers
// Base layer
layer("base", #{
    // Example: Space as layer toggle
    "KEY_SPACE": tap_hold("KEY_SPACE", layer_toggle("lower"), 200),
});

// Lower layer - symbols and numbers
layer("lower", #{
    "KEY_A": simple("KEY_1"),
    "KEY_S": simple("KEY_2"),
    "KEY_D": simple("KEY_3"),
});
"#
        .to_string()
    }

    /// Activate a profile with hot-reload.
    pub fn activate(&mut self, name: &str) -> Result<ActivationResult, ProfileError> {
        // Acquire activation lock to serialize concurrent activations
        let _lock = self.activation_lock.lock().unwrap();

        let start = Instant::now();

        // Get profile metadata
        let profile = self
            .profiles
            .get(name)
            .ok_or_else(|| ProfileError::NotFound(name.to_string()))?
            .clone();

        // Compile and reload
        let (compile_time, reload_time) = match self.compile_and_reload(name, &profile) {
            Ok(times) => times,
            Err((compile_time, e)) => {
                return Ok(ActivationResult {
                    compile_time_ms: compile_time,
                    reload_time_ms: 0,
                    success: false,
                    error: Some(e.to_string()),
                });
            }
        };

        log::info!(
            "Profile '{}' activated in {}ms (compile: {}ms, reload: {}ms)",
            name,
            start.elapsed().as_millis(),
            compile_time,
            reload_time
        );

        Ok(ActivationResult {
            compile_time_ms: compile_time,
            reload_time_ms: reload_time,
            success: true,
            error: None,
        })
    }

    /// Compile and reload a profile.
    fn compile_and_reload(
        &self,
        name: &str,
        profile: &ProfileMetadata,
    ) -> Result<(u64, u64), (u64, ProfileError)> {
        // Compile .rhai → .krx with timeout
        let compile_start = Instant::now();

        if let Err(e) = self.compile_with_timeout(&profile.rhai_path, &profile.krx_path) {
            return Err((compile_start.elapsed().as_millis() as u64, e));
        }

        let compile_time = compile_start.elapsed().as_millis() as u64;

        // Atomic swap
        let reload_start = Instant::now();
        *self.active_profile.write().unwrap() = Some(name.to_string());
        let reload_time = reload_start.elapsed().as_millis() as u64;

        Ok((compile_time, reload_time))
    }

    /// Compile with timeout.
    fn compile_with_timeout(&self, rhai_path: &Path, krx_path: &Path) -> Result<(), ProfileError> {
        // For now, use keyrx_compiler directly
        // In production, this would use timeout mechanism
        keyrx_compiler::compile_file(rhai_path, krx_path)
            .map_err(|e| ProfileError::CompilationFailed(e.to_string()))?;

        Ok(())
    }

    /// Delete a profile.
    pub fn delete(&mut self, name: &str) -> Result<(), ProfileError> {
        let profile = self
            .profiles
            .get(name)
            .ok_or_else(|| ProfileError::NotFound(name.to_string()))?
            .clone();

        // Check if this is the active profile
        let active = self.active_profile.read().unwrap();
        if active.as_deref() == Some(name) {
            drop(active);
            *self.active_profile.write().unwrap() = None;
        }

        // Delete both .rhai and .krx files
        if profile.rhai_path.exists() {
            fs::remove_file(&profile.rhai_path)?;
        }
        if profile.krx_path.exists() {
            fs::remove_file(&profile.krx_path)?;
        }

        self.profiles.remove(name);

        Ok(())
    }

    /// Duplicate a profile.
    pub fn duplicate(&mut self, src: &str, dest: &str) -> Result<ProfileMetadata, ProfileError> {
        Self::validate_name(dest)?;

        if self.profiles.len() >= MAX_PROFILES {
            return Err(ProfileError::ProfileLimitExceeded);
        }

        let src_profile = self
            .profiles
            .get(src)
            .ok_or_else(|| ProfileError::NotFound(src.to_string()))?
            .clone();

        let dest_rhai = self
            .config_dir
            .join("profiles")
            .join(format!("{}.rhai", dest));
        if dest_rhai.exists() {
            return Err(ProfileError::AlreadyExists(dest.to_string()));
        }

        fs::copy(&src_profile.rhai_path, &dest_rhai)?;

        let metadata = self.load_profile_metadata(dest)?;
        self.profiles.insert(dest.to_string(), metadata.clone());

        Ok(metadata)
    }

    /// Export a profile to a file.
    pub fn export(&self, name: &str, dest: &Path) -> Result<(), ProfileError> {
        let profile = self
            .profiles
            .get(name)
            .ok_or_else(|| ProfileError::NotFound(name.to_string()))?;

        fs::copy(&profile.rhai_path, dest)?;
        Ok(())
    }

    /// Import a profile from a file.
    pub fn import(&mut self, src: &Path, name: &str) -> Result<ProfileMetadata, ProfileError> {
        Self::validate_name(name)?;

        if self.profiles.len() >= MAX_PROFILES {
            return Err(ProfileError::ProfileLimitExceeded);
        }

        let dest_rhai = self
            .config_dir
            .join("profiles")
            .join(format!("{}.rhai", name));
        if dest_rhai.exists() {
            return Err(ProfileError::AlreadyExists(name.to_string()));
        }

        fs::copy(src, &dest_rhai)?;

        let metadata = self.load_profile_metadata(name)?;
        self.profiles.insert(name.to_string(), metadata.clone());

        Ok(metadata)
    }

    /// List all profiles.
    pub fn list(&self) -> Vec<&ProfileMetadata> {
        self.profiles.values().collect()
    }

    /// Get the currently active profile name.
    pub fn get_active(&self) -> Option<String> {
        self.active_profile.read().unwrap().clone()
    }

    /// Get profile metadata by name.
    pub fn get(&self, name: &str) -> Option<&ProfileMetadata> {
        self.profiles.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_manager() -> (TempDir, ProfileManager) {
        let temp_dir = TempDir::new().unwrap();
        let manager = ProfileManager::new(temp_dir.path().to_path_buf()).unwrap();
        (temp_dir, manager)
    }

    #[test]
    fn test_create_blank_profile() {
        let (_temp, mut manager) = setup_test_manager();

        let result = manager.create("test-profile", ProfileTemplate::Blank);
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.name, "test-profile");
        assert!(metadata.rhai_path.exists());
    }

    #[test]
    fn test_create_qmk_profile() {
        let (_temp, mut manager) = setup_test_manager();

        let result = manager.create("qmk-test", ProfileTemplate::QmkLayers);
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.name, "qmk-test");
        assert!(metadata.layer_count > 1);
    }

    #[test]
    fn test_profile_name_validation() {
        assert!(ProfileManager::validate_name("valid-name_123").is_ok());
        assert!(ProfileManager::validate_name("").is_err());
        assert!(ProfileManager::validate_name(&"a".repeat(100)).is_err());
        assert!(ProfileManager::validate_name("invalid name!").is_err());
    }

    #[test]
    fn test_duplicate_profile() {
        let (_temp, mut manager) = setup_test_manager();

        manager.create("original", ProfileTemplate::Blank).unwrap();
        let result = manager.duplicate("original", "copy");

        assert!(result.is_ok());
        assert!(manager.get("copy").is_some());
    }

    #[test]
    fn test_delete_profile() {
        let (_temp, mut manager) = setup_test_manager();

        manager.create("to-delete", ProfileTemplate::Blank).unwrap();
        assert!(manager.get("to-delete").is_some());

        manager.delete("to-delete").unwrap();
        assert!(manager.get("to-delete").is_none());
    }

    #[test]
    fn test_list_profiles() {
        let (_temp, mut manager) = setup_test_manager();

        manager.create("profile1", ProfileTemplate::Blank).unwrap();
        manager
            .create("profile2", ProfileTemplate::QmkLayers)
            .unwrap();

        let profiles = manager.list();
        assert_eq!(profiles.len(), 2);
    }

    #[test]
    fn test_profile_limit() {
        let (_temp, mut manager) = setup_test_manager();

        // Create MAX_PROFILES profiles
        for i in 0..MAX_PROFILES {
            manager
                .create(&format!("profile{}", i), ProfileTemplate::Blank)
                .unwrap();
        }

        // Next one should fail
        let result = manager.create("overflow", ProfileTemplate::Blank);
        assert!(matches!(result, Err(ProfileError::ProfileLimitExceeded)));
    }

    #[test]
    fn test_export_import() {
        let (_temp, mut manager) = setup_test_manager();

        manager.create("original", ProfileTemplate::Blank).unwrap();

        let export_path = _temp.path().join("exported.rhai");
        manager.export("original", &export_path).unwrap();

        assert!(export_path.exists());

        manager.import(&export_path, "imported").unwrap();
        assert!(manager.get("imported").is_some());
    }

    #[test]
    fn test_get_active_profile() {
        let (_temp, mut manager) = setup_test_manager();

        assert!(manager.get_active().is_none());

        manager.create("test", ProfileTemplate::Blank).unwrap();

        // Note: activate() requires compilation which we can't do in unit tests
        // without a real compiler setup, so we just test the get_active() method
    }

    #[test]
    fn test_scan_profiles() {
        let temp_dir = TempDir::new().unwrap();
        let profiles_dir = temp_dir.path().join("profiles");
        fs::create_dir_all(&profiles_dir).unwrap();

        // Create some .rhai files manually
        fs::write(profiles_dir.join("test1.rhai"), "layer(\"base\", #{});").unwrap();
        fs::write(profiles_dir.join("test2.rhai"), "layer(\"base\", #{});").unwrap();

        let manager = ProfileManager::new(temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(manager.list().len(), 2);
    }

    #[test]
    fn test_profile_already_exists() {
        let (_temp, mut manager) = setup_test_manager();

        manager.create("existing", ProfileTemplate::Blank).unwrap();
        let result = manager.create("existing", ProfileTemplate::Blank);

        assert!(matches!(result, Err(ProfileError::AlreadyExists(_))));
    }

    #[test]
    fn test_duplicate_nonexistent_profile() {
        let (_temp, mut manager) = setup_test_manager();

        let result = manager.duplicate("nonexistent", "copy");
        assert!(matches!(result, Err(ProfileError::NotFound(_))));
    }

    #[test]
    fn test_duplicate_to_existing_name() {
        let (_temp, mut manager) = setup_test_manager();

        manager.create("original", ProfileTemplate::Blank).unwrap();
        manager.create("existing", ProfileTemplate::Blank).unwrap();

        let result = manager.duplicate("original", "existing");
        assert!(matches!(result, Err(ProfileError::AlreadyExists(_))));
    }

    #[test]
    fn test_delete_nonexistent_profile() {
        let (_temp, mut manager) = setup_test_manager();

        let result = manager.delete("nonexistent");
        assert!(matches!(result, Err(ProfileError::NotFound(_))));
    }

    #[test]
    fn test_delete_active_profile() {
        let (_temp, mut manager) = setup_test_manager();

        manager
            .create("active-profile", ProfileTemplate::Blank)
            .unwrap();

        // Simulate activating the profile by setting it directly
        *manager.active_profile.write().unwrap() = Some("active-profile".to_string());

        assert_eq!(manager.get_active(), Some("active-profile".to_string()));

        // Delete the active profile
        manager.delete("active-profile").unwrap();

        // Active profile should be cleared
        assert!(manager.get_active().is_none());
        assert!(manager.get("active-profile").is_none());
    }

    #[test]
    fn test_export_nonexistent_profile() {
        let (_temp, manager) = setup_test_manager();

        let export_path = _temp.path().join("export.rhai");
        let result = manager.export("nonexistent", &export_path);

        assert!(matches!(result, Err(ProfileError::NotFound(_))));
    }

    #[test]
    fn test_import_to_existing_name() {
        let (_temp, mut manager) = setup_test_manager();

        manager.create("existing", ProfileTemplate::Blank).unwrap();

        let import_path = _temp.path().join("import.rhai");
        fs::write(&import_path, "layer(\"base\", #{});").unwrap();

        let result = manager.import(&import_path, "existing");
        assert!(matches!(result, Err(ProfileError::AlreadyExists(_))));
    }

    #[test]
    fn test_import_invalid_name() {
        let (_temp, mut manager) = setup_test_manager();

        let import_path = _temp.path().join("import.rhai");
        fs::write(&import_path, "layer(\"base\", #{});").unwrap();

        let result = manager.import(&import_path, "invalid name!");
        assert!(matches!(result, Err(ProfileError::InvalidName(_))));
    }

    #[test]
    fn test_get_nonexistent_profile() {
        let (_temp, manager) = setup_test_manager();

        assert!(manager.get("nonexistent").is_none());
    }

    #[test]
    fn test_layer_count_heuristic() {
        let (_temp, mut manager) = setup_test_manager();

        // Create profile with multiple layers
        let multi_layer = r#"
layer("base", #{});
layer("layer1", #{});
layer("layer2", #{});
"#;

        let profiles_dir = _temp.path().join("profiles");
        let multi_path = profiles_dir.join("multi.rhai");
        fs::write(&multi_path, multi_layer).unwrap();

        let metadata = manager.load_profile_metadata("multi").unwrap();
        assert_eq!(metadata.layer_count, 3);

        manager.scan_profiles().unwrap();
        let profile = manager.get("multi").unwrap();
        assert_eq!(profile.layer_count, 3);
    }

    #[test]
    fn test_layer_count_minimum() {
        let (_temp, manager) = setup_test_manager();

        // Create profile with no explicit layers
        let no_layers = "// Empty config";

        let profiles_dir = _temp.path().join("profiles");
        let empty_path = profiles_dir.join("empty.rhai");
        fs::write(&empty_path, no_layers).unwrap();

        let metadata = manager.load_profile_metadata("empty").unwrap();
        // Should default to at least 1 layer
        assert_eq!(metadata.layer_count, 1);
    }

    #[test]
    fn test_validate_name_edge_cases() {
        // Valid names
        assert!(ProfileManager::validate_name("a").is_ok());
        assert!(ProfileManager::validate_name("A").is_ok());
        assert!(ProfileManager::validate_name("0").is_ok());
        assert!(ProfileManager::validate_name("a-b").is_ok());
        assert!(ProfileManager::validate_name("a_b").is_ok());
        assert!(ProfileManager::validate_name("a-b_c123").is_ok());
        assert!(ProfileManager::validate_name(&"a".repeat(32)).is_ok());

        // Invalid names
        assert!(ProfileManager::validate_name("").is_err());
        assert!(ProfileManager::validate_name(&"a".repeat(33)).is_err());
        assert!(ProfileManager::validate_name("a b").is_err());
        assert!(ProfileManager::validate_name("a!b").is_err());
        assert!(ProfileManager::validate_name("a@b").is_err());
        assert!(ProfileManager::validate_name("a.b").is_err());
        assert!(ProfileManager::validate_name("a/b").is_err());
    }

    #[test]
    fn test_profile_limit_with_duplicate() {
        let (_temp, mut manager) = setup_test_manager();

        // Create MAX_PROFILES - 1 profiles
        for i in 0..(MAX_PROFILES - 1) {
            manager
                .create(&format!("profile{}", i), ProfileTemplate::Blank)
                .unwrap();
        }

        // Create one more profile
        manager.create("last", ProfileTemplate::Blank).unwrap();

        // Now at MAX_PROFILES, duplicate should fail
        let result = manager.duplicate("last", "overflow");
        assert!(matches!(result, Err(ProfileError::ProfileLimitExceeded)));
    }

    #[test]
    fn test_profile_limit_with_import() {
        let (_temp, mut manager) = setup_test_manager();

        // Create MAX_PROFILES profiles
        for i in 0..MAX_PROFILES {
            manager
                .create(&format!("profile{}", i), ProfileTemplate::Blank)
                .unwrap();
        }

        // Try to import when at limit
        let import_path = _temp.path().join("import.rhai");
        fs::write(&import_path, "layer(\"base\", #{});").unwrap();

        let result = manager.import(&import_path, "overflow");
        assert!(matches!(result, Err(ProfileError::ProfileLimitExceeded)));
    }

    #[test]
    fn test_scan_after_manual_file_creation() {
        let (_temp, mut manager) = setup_test_manager();

        // Create profile through manager
        manager.create("profile1", ProfileTemplate::Blank).unwrap();
        assert_eq!(manager.list().len(), 1);

        // Manually create another profile file
        let profiles_dir = _temp.path().join("profiles");
        fs::write(profiles_dir.join("manual.rhai"), "layer(\"base\", #{});").unwrap();

        // Scan should find both
        manager.scan_profiles().unwrap();
        assert_eq!(manager.list().len(), 2);
        assert!(manager.get("manual").is_some());
    }

    #[test]
    fn test_scan_ignores_non_rhai_files() {
        let temp_dir = TempDir::new().unwrap();
        let profiles_dir = temp_dir.path().join("profiles");
        fs::create_dir_all(&profiles_dir).unwrap();

        // Create various file types
        fs::write(profiles_dir.join("profile.rhai"), "layer(\"base\", #{});").unwrap();
        fs::write(profiles_dir.join("config.toml"), "key = \"value\"").unwrap();
        fs::write(profiles_dir.join("data.json"), "{}").unwrap();
        fs::write(profiles_dir.join("README.md"), "# Profiles").unwrap();

        let manager = ProfileManager::new(temp_dir.path().to_path_buf()).unwrap();

        // Should only find the .rhai file
        assert_eq!(manager.list().len(), 1);
        assert!(manager.get("profile").is_some());
    }

    #[test]
    fn test_metadata_preserves_name() {
        let (_temp, mut manager) = setup_test_manager();

        manager.create("test-name", ProfileTemplate::Blank).unwrap();

        let profile = manager.get("test-name").unwrap();
        assert_eq!(profile.name, "test-name");
    }

    #[test]
    fn test_metadata_paths() {
        let (_temp, mut manager) = setup_test_manager();

        manager.create("test", ProfileTemplate::Blank).unwrap();

        let profile = manager.get("test").unwrap();
        assert!(profile.rhai_path.ends_with("profiles/test.rhai"));
        assert!(profile.krx_path.ends_with("profiles/test.krx"));
    }

    #[test]
    fn test_rescan_after_delete() {
        let (_temp, mut manager) = setup_test_manager();

        manager.create("profile1", ProfileTemplate::Blank).unwrap();
        manager.create("profile2", ProfileTemplate::Blank).unwrap();
        assert_eq!(manager.list().len(), 2);

        manager.delete("profile1").unwrap();
        assert_eq!(manager.list().len(), 1);

        // Rescan should maintain correct state
        manager.scan_profiles().unwrap();
        assert_eq!(manager.list().len(), 1);
        assert!(manager.get("profile1").is_none());
        assert!(manager.get("profile2").is_some());
    }

    #[test]
    fn test_templates_generate_valid_content() {
        let blank = ProfileManager::generate_blank_template();
        assert!(blank.contains("layer("));
        assert!(blank.contains("base"));

        let qmk = ProfileManager::generate_qmk_template();
        assert!(qmk.contains("layer("));
        assert!(qmk.contains("base"));
        assert!(qmk.contains("lower"));
        assert!(qmk.contains("tap_hold"));
    }

    #[test]
    fn test_new_creates_config_dir() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("new_config");

        // Directory doesn't exist yet
        assert!(!config_path.exists());

        let _manager = ProfileManager::new(config_path.clone()).unwrap();

        // Directory and profiles subdirectory should be created
        assert!(config_path.exists());
        assert!(config_path.join("profiles").exists());
    }

    #[test]
    fn test_new_with_existing_config_dir() {
        let temp_dir = TempDir::new().unwrap();

        // Create config directory first
        fs::create_dir_all(temp_dir.path()).unwrap();

        let _manager = ProfileManager::new(temp_dir.path().to_path_buf()).unwrap();

        // Should work fine with existing directory
        assert!(temp_dir.path().exists());
        assert!(temp_dir.path().join("profiles").exists());
    }

    #[test]
    fn test_delete_removes_both_files() {
        let (_temp, mut manager) = setup_test_manager();

        manager.create("test", ProfileTemplate::Blank).unwrap();

        // Get paths before deletion
        let profile = manager.get("test").unwrap().clone();
        let rhai_path = profile.rhai_path.clone();
        let krx_path = profile.krx_path.clone();

        // Manually create .krx file to test deletion
        fs::write(&krx_path, b"dummy krx content").unwrap();

        assert!(rhai_path.exists());
        assert!(krx_path.exists());

        manager.delete("test").unwrap();

        // Both files should be deleted
        assert!(!rhai_path.exists());
        assert!(!krx_path.exists());
    }

    #[test]
    fn test_delete_with_missing_krx() {
        let (_temp, mut manager) = setup_test_manager();

        manager.create("test", ProfileTemplate::Blank).unwrap();

        let profile = manager.get("test").unwrap().clone();
        assert!(profile.rhai_path.exists());
        // .krx doesn't exist yet

        // Delete should work even if .krx doesn't exist
        manager.delete("test").unwrap();

        assert!(!profile.rhai_path.exists());
        assert!(manager.get("test").is_none());
    }

    #[test]
    fn test_load_profile_metadata_nonexistent() {
        let (_temp, manager) = setup_test_manager();

        let result = manager.load_profile_metadata("nonexistent");
        assert!(matches!(result, Err(ProfileError::NotFound(_))));
    }

    #[test]
    fn test_count_layers_multiple() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        let content = r#"
layer("base", #{});
layer("layer1", #{});
layer("layer2", #{});
layer("layer3", #{});
"#;
        fs::write(&test_file, content).unwrap();

        let count = ProfileManager::count_layers(&test_file).unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn test_count_layers_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("empty.rhai");

        fs::write(&test_file, "").unwrap();

        let count = ProfileManager::count_layers(&test_file).unwrap();
        // Should default to at least 1
        assert_eq!(count, 1);
    }

    #[test]
    fn test_list_returns_all_profiles() {
        let (_temp, mut manager) = setup_test_manager();

        manager.create("profile1", ProfileTemplate::Blank).unwrap();
        manager
            .create("profile2", ProfileTemplate::QmkLayers)
            .unwrap();
        manager.create("profile3", ProfileTemplate::Blank).unwrap();

        let profiles = manager.list();
        assert_eq!(profiles.len(), 3);

        // Verify names are present
        let names: Vec<&str> = profiles.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"profile1"));
        assert!(names.contains(&"profile2"));
        assert!(names.contains(&"profile3"));
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ProfileManager::new(temp_dir.path().to_path_buf()).unwrap();

        // Should work with empty profiles directory
        assert_eq!(manager.list().len(), 0);
    }

    #[test]
    fn test_duplicate_preserves_content() {
        let (_temp, mut manager) = setup_test_manager();

        manager
            .create("original", ProfileTemplate::QmkLayers)
            .unwrap();

        let original = manager.get("original").unwrap();
        let original_content = fs::read_to_string(&original.rhai_path).unwrap();

        manager.duplicate("original", "copy").unwrap();

        let copy = manager.get("copy").unwrap();
        let copy_content = fs::read_to_string(&copy.rhai_path).unwrap();

        // Content should be identical
        assert_eq!(original_content, copy_content);
    }
}
