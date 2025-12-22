//! Linux-specific device enumeration using evdev.
//!
//! This module scans `/dev/input/event*` devices and identifies keyboards
//! based on their capabilities (presence of alphabetic keys).

use std::fs;
use std::path::Path;

use evdev::{Device, EventType, Key};
use log::debug;

use super::{DiscoveryError, KeyboardInfo};

/// Required alphabetic keys that a keyboard must have.
/// If a device has EV_KEY capability and these keys, it's considered a keyboard.
const REQUIRED_KEYS: &[Key] = &[
    Key::KEY_A,
    Key::KEY_B,
    Key::KEY_C,
    Key::KEY_D,
    Key::KEY_E,
    Key::KEY_F,
    Key::KEY_G,
    Key::KEY_H,
    Key::KEY_I,
    Key::KEY_J,
    Key::KEY_K,
    Key::KEY_L,
    Key::KEY_M,
    Key::KEY_N,
    Key::KEY_O,
    Key::KEY_P,
    Key::KEY_Q,
    Key::KEY_R,
    Key::KEY_S,
    Key::KEY_T,
    Key::KEY_U,
    Key::KEY_V,
    Key::KEY_W,
    Key::KEY_X,
    Key::KEY_Y,
    Key::KEY_Z,
];

/// Minimum number of required keys that must be present to consider a device a keyboard.
/// This threshold helps filter out devices that might have a few key events but aren't
/// full keyboards (like power buttons or multimedia remotes).
const MIN_REQUIRED_KEYS: usize = 20;

/// Checks if a device has keyboard capabilities.
///
/// A device is considered a keyboard if it:
/// 1. Supports the EV_KEY event type
/// 2. Has at least `MIN_REQUIRED_KEYS` of the required alphabetic keys
///
/// This filtering excludes mice, touchpads, power buttons, and other input
/// devices that may report some key events but are not keyboards.
fn is_keyboard(device: &Device) -> bool {
    // Must support key events
    let supported_events = device.supported_events();
    if !supported_events.contains(EventType::KEY) {
        return false;
    }

    // Check for alphabetic keys
    let Some(supported_keys) = device.supported_keys() else {
        return false;
    };

    let key_count = REQUIRED_KEYS
        .iter()
        .filter(|key| supported_keys.contains(**key))
        .count();

    key_count >= MIN_REQUIRED_KEYS
}

/// Enumerates all keyboard devices on the system.
///
/// This function scans `/dev/input/event*` devices and returns information
/// about each device that appears to be a keyboard (has EV_KEY capability
/// with alphabetic keys).
///
/// # Returns
///
/// * `Ok(Vec<KeyboardInfo>)` - List of discovered keyboard devices
/// * `Err(DiscoveryError::Io)` - Failed to read `/dev/input` directory
///
/// # Permissions
///
/// This function attempts to open each device to read its capabilities.
/// Devices that cannot be opened (due to permissions) are skipped with
/// a debug log message. To enumerate all devices, the user typically needs:
/// - Root access, OR
/// - Membership in the `input` group, OR
/// - Appropriate udev rules
///
/// # Example
///
/// ```no_run
/// use keyrx_daemon::device_manager::enumerate_keyboards;
///
/// match enumerate_keyboards() {
///     Ok(keyboards) => {
///         println!("Found {} keyboard(s):", keyboards.len());
///         for kb in keyboards {
///             println!("  {} ({})", kb.name, kb.path.display());
///         }
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn enumerate_keyboards() -> Result<Vec<KeyboardInfo>, DiscoveryError> {
    let input_dir = Path::new("/dev/input");

    // Read directory entries
    let entries = fs::read_dir(input_dir)?;

    let mut keyboards = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                debug!("Failed to read directory entry: {}", e);
                continue;
            }
        };

        let path = entry.path();

        // Only consider event* devices
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) if name.starts_with("event") => name,
            _ => continue,
        };

        // Try to open the device
        let device = match Device::open(&path) {
            Ok(d) => d,
            Err(e) => {
                // Permission denied is common for devices not accessible to user
                debug!("Skipping {} ({}): {}", file_name, path.display(), e);
                continue;
            }
        };

        // Check if it's a keyboard
        if !is_keyboard(&device) {
            debug!("Skipping {} (not a keyboard)", file_name);
            continue;
        }

        // Collect device information
        let name = device.name().unwrap_or("Unknown Device").to_string();
        let serial = device
            .unique_name()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());
        let phys = device
            .physical_path()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());

        debug!("Found keyboard: {} at {}", name, path.display());

        keyboards.push(KeyboardInfo {
            path,
            name,
            serial,
            phys,
        });
    }

    // Sort by path for consistent ordering
    keyboards.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(keyboards)
}

/// Opens a device by path and returns it if it's a valid keyboard.
///
/// # Arguments
///
/// * `path` - Path to the device node (e.g., `/dev/input/event0`)
///
/// # Returns
///
/// * `Ok(Some(KeyboardInfo))` - Device is a keyboard
/// * `Ok(None)` - Device exists but is not a keyboard
/// * `Err(DiscoveryError)` - Failed to access device
#[allow(dead_code)] // Will be used in task #12 (DeviceManager)
pub fn open_keyboard(path: &Path) -> Result<Option<KeyboardInfo>, DiscoveryError> {
    let device = Device::open(path).map_err(|e| {
        let kind = e.kind();
        match kind {
            std::io::ErrorKind::NotFound => DiscoveryError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("device not found: {}", path.display()),
            )),
            std::io::ErrorKind::PermissionDenied => DiscoveryError::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("permission denied: {}", path.display()),
            )),
            _ => DiscoveryError::Io(e),
        }
    })?;

    if !is_keyboard(&device) {
        return Ok(None);
    }

    let name = device.name().unwrap_or("Unknown Device").to_string();
    let serial = device
        .unique_name()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());
    let phys = device
        .physical_path()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());

    Ok(Some(KeyboardInfo {
        path: path.to_path_buf(),
        name,
        serial,
        phys,
    }))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_required_keys_constant() {
        // Verify we have all 26 letters
        assert_eq!(REQUIRED_KEYS.len(), 26);
    }

    #[test]
    fn test_min_required_keys() {
        // Threshold should be reasonable (most of the alphabet)
        assert!(MIN_REQUIRED_KEYS >= 20);
        assert!(MIN_REQUIRED_KEYS <= REQUIRED_KEYS.len());
    }

    // Integration tests that require real devices
    #[test]
    #[ignore = "Requires access to /dev/input devices"]
    fn test_enumerate_keyboards_real_devices() {
        let result = enumerate_keyboards();
        assert!(result.is_ok(), "Should not error on enumeration");

        let keyboards = result.unwrap();
        println!("Found {} keyboard(s):", keyboards.len());
        for kb in &keyboards {
            println!("  Name: {}", kb.name);
            println!("  Path: {}", kb.path.display());
            if let Some(ref serial) = kb.serial {
                println!("  Serial: {}", serial);
            }
            if let Some(ref phys) = kb.phys {
                println!("  Phys: {}", phys);
            }
            println!();
        }
    }

    #[test]
    #[ignore = "Requires access to /dev/input devices"]
    fn test_open_keyboard_event0() {
        let path = Path::new("/dev/input/event0");
        let result = open_keyboard(path);

        match result {
            Ok(Some(kb)) => {
                println!("event0 is a keyboard: {}", kb.name);
            }
            Ok(None) => {
                println!("event0 exists but is not a keyboard");
            }
            Err(e) => {
                println!("Failed to open event0: {}", e);
            }
        }
    }

    #[test]
    fn test_open_keyboard_nonexistent() {
        let path = Path::new("/dev/input/event99999");
        let result = open_keyboard(path);

        assert!(matches!(result, Err(DiscoveryError::Io(_))));
    }

    #[test]
    fn test_keyboard_info_fields() {
        let info = KeyboardInfo {
            path: PathBuf::from("/dev/input/event0"),
            name: "AT Translated Set 2 keyboard".to_string(),
            serial: Some("0000:00:00".to_string()),
            phys: Some("isa0060/serio0/input0".to_string()),
        };

        assert_eq!(info.path, PathBuf::from("/dev/input/event0"));
        assert_eq!(info.name, "AT Translated Set 2 keyboard");
        assert_eq!(info.serial.as_deref(), Some("0000:00:00"));
        assert_eq!(info.phys.as_deref(), Some("isa0060/serio0/input0"));
    }
}
