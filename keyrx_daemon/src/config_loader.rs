//! Configuration file loading module.
//!
//! This module provides functionality to load and validate .krx binary configuration files.

use std::path::Path;

use keyrx_compiler::error::DeserializeError;
use keyrx_core::config::ConfigRoot;
use thiserror::Error;

/// Errors that can occur during configuration loading.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// I/O error while reading the configuration file.
    ///
    /// This error occurs when:
    /// - The file does not exist
    /// - The process lacks read permissions
    /// - The file is locked by another process
    /// - The filesystem is unavailable
    #[error("Failed to read configuration file: {0}")]
    Io(#[from] std::io::Error),

    /// Deserialization error while parsing the configuration file.
    ///
    /// This error occurs when:
    /// - Invalid magic bytes (file is not a .krx file)
    /// - Version mismatch (incompatible .krx format version)
    /// - Hash mismatch (data corruption detected)
    /// - Invalid rkyv archive structure
    #[error("Failed to deserialize configuration: {0}")]
    Deserialize(DeserializeError),
}

impl From<DeserializeError> for ConfigError {
    fn from(err: DeserializeError) -> Self {
        ConfigError::Deserialize(err)
    }
}

/// Loads and validates a .krx configuration file.
///
/// This function:
/// 1. Reads the file from disk
/// 2. Validates the .krx file format (magic bytes, version, hash)
/// 3. Deserializes the configuration using rkyv
///
/// # Arguments
///
/// * `path` - Path to the .krx configuration file
///
/// # Returns
///
/// Returns a zero-copy reference to the archived ConfigRoot on success.
///
/// # Errors
///
/// Returns `ConfigError::Io` if:
/// - The file does not exist
/// - The process lacks read permissions
/// - An I/O error occurs while reading
///
/// Returns `ConfigError::Deserialize` if:
/// - The file has invalid magic bytes (not a .krx file)
/// - The .krx format version is incompatible
/// - The hash does not match (data corruption)
/// - The rkyv archive structure is invalid
///
/// # Examples
///
/// ```no_run
/// use keyrx_daemon::config_loader::load_config;
///
/// // Load configuration from a file
/// let config = load_config("config.krx")?;
///
/// // Access devices (ArchivedVec requires .as_slice() for iteration)
/// for device in config.devices.as_slice() {
///     println!("Device pattern: {}", device.identifier.pattern);
/// }
/// # Ok::<(), keyrx_daemon::config_loader::ConfigError>(())
/// ```
pub fn load_config<P: AsRef<Path>>(
    path: P,
) -> Result<&'static rkyv::Archived<ConfigRoot>, ConfigError> {
    // Read file bytes
    let bytes = std::fs::read(path)?;

    // Leak the bytes to get a 'static lifetime
    // This is necessary because rkyv::archived_root requires 'static lifetime
    // The memory will live for the entire program duration
    let static_bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());

    // Deserialize and validate the .krx file
    let config = keyrx_compiler::serialize::deserialize(static_bytes)?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyrx_compiler::serialize::serialize;
    use keyrx_core::config::{
        DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
    };
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_config() -> ConfigRoot {
        ConfigRoot {
            version: Version::current(),
            devices: vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: "Test Device".to_string(),
                },
                mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
            }],
            metadata: Metadata {
                compilation_timestamp: 1234567890,
                compiler_version: "1.0.0".to_string(),
                source_hash: "test_hash".to_string(),
            },
        }
    }

    #[test]
    fn test_load_valid_config() {
        // Create a valid .krx file
        let config = create_test_config();
        let bytes = serialize(&config).expect("Serialization failed");

        // Write to temporary file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(&bytes)
            .expect("Failed to write to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        // Load the configuration
        let result = load_config(temp_file.path());
        assert!(result.is_ok());

        let loaded = result.unwrap();
        assert_eq!(loaded.devices.len(), 1);
        assert_eq!(loaded.devices[0].mappings.len(), 1);
    }

    #[test]
    fn test_load_missing_file() {
        let result = load_config("/nonexistent/path/to/config.krx");
        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::Io(_))));
    }

    #[test]
    fn test_load_corrupted_magic() {
        // Create a file with invalid magic bytes
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(b"INVALID DATA")
            .expect("Failed to write to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        let result = load_config(temp_file.path());
        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::Deserialize(_))));
    }

    #[test]
    fn test_load_corrupted_hash() {
        // Create a valid .krx file
        let config = create_test_config();
        let mut bytes = serialize(&config).expect("Serialization failed");

        // Corrupt the hash (bytes 8-40 are the hash)
        bytes[8] = !bytes[8];

        // Write to temporary file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(&bytes)
            .expect("Failed to write to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        let result = load_config(temp_file.path());
        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::Deserialize(_))));
    }

    #[test]
    fn test_config_error_display() {
        // Test Io error display
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let config_error = ConfigError::Io(io_error);
        let display = format!("{}", config_error);
        assert!(display.contains("Failed to read configuration file"));

        // Test Deserialize error display
        let deserialize_error = DeserializeError::InvalidMagic {
            expected: [0x4B, 0x52, 0x58, 0x0A],
            got: [0x00, 0x00, 0x00, 0x00],
        };
        let config_error = ConfigError::Deserialize(deserialize_error);
        let display = format!("{}", config_error);
        assert!(display.contains("Failed to deserialize configuration"));
    }
}
