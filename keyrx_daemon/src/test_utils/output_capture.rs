//! Output capture for reading daemon's virtual keyboard events.
//!
//! This module provides [`OutputCapture`] for finding and reading events
//! from the daemon's virtual output keyboard device.
//!
//! # Usage
//!
//! ```ignore
//! use keyrx_daemon::test_utils::OutputCapture;
//! use std::time::Duration;
//!
//! // Find the daemon's output device (polls until found or timeout)
//! let capture = OutputCapture::find_by_name(
//!     "keyrx Virtual Keyboard",
//!     Duration::from_secs(5)
//! )?;
//!
//! // Device path is available for debugging
//! println!("Found device at: {}", capture.device_path());
//! ```
//!
//! # Requirements
//!
//! - Linux with evdev support
//! - Read access to `/dev/input/event*` devices (typically requires `input` group)

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use evdev::Device;

use super::VirtualDeviceError;

/// Polling interval when waiting for a device to appear.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Captures output events from the daemon's virtual keyboard.
///
/// Finds and opens the daemon's output device by name, then provides
/// methods for reading events with timeout handling.
///
/// # Device Discovery
///
/// The `find_by_name` method polls `/dev/input/event*` devices until one
/// matching the specified name is found. This handles the race condition
/// where the daemon may not have created its output device yet.
///
/// # Example
///
/// ```ignore
/// use keyrx_daemon::test_utils::OutputCapture;
/// use std::time::Duration;
///
/// // Wait up to 5 seconds for the daemon's output device
/// let capture = OutputCapture::find_by_name(
///     "keyrx Virtual Keyboard",
///     Duration::from_secs(5)
/// )?;
///
/// println!("Capturing from: {}", capture.name());
/// ```
pub struct OutputCapture {
    /// The evdev device handle for reading events.
    device: Device,
    /// Name of the device (as reported by evdev).
    name: String,
    /// Path to the device node (e.g., /dev/input/event5).
    device_path: PathBuf,
}

impl std::fmt::Debug for OutputCapture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputCapture")
            .field("name", &self.name)
            .field("device_path", &self.device_path)
            .finish_non_exhaustive()
    }
}

impl OutputCapture {
    /// Finds and opens an output device by name.
    ///
    /// Polls `/dev/input/event*` devices until one with a matching name is found
    /// or the timeout expires. This handles the race condition where the daemon
    /// may not have created its virtual output device yet.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the device to find (exact match)
    /// * `timeout` - Maximum time to wait for the device
    ///
    /// # Returns
    ///
    /// An `OutputCapture` instance connected to the device, or an error.
    ///
    /// # Errors
    ///
    /// - [`VirtualDeviceError::NotFound`] if device not found within timeout
    /// - [`VirtualDeviceError::PermissionDenied`] if device is not accessible
    ///
    /// # Example
    ///
    /// ```ignore
    /// use keyrx_daemon::test_utils::OutputCapture;
    /// use std::time::Duration;
    ///
    /// // Find the daemon's output device
    /// let capture = OutputCapture::find_by_name(
    ///     "keyrx Virtual Keyboard",
    ///     Duration::from_secs(5)
    /// )?;
    ///
    /// println!("Found: {} at {}", capture.name(), capture.device_path());
    /// ```
    pub fn find_by_name(name: &str, timeout: Duration) -> Result<Self, VirtualDeviceError> {
        let start = Instant::now();
        let timeout_ms = timeout.as_millis() as u64;

        loop {
            // Try to find the device
            match Self::try_find_device(name) {
                Ok(Some(capture)) => return Ok(capture),
                Ok(None) => {
                    // Device not found yet, check timeout
                    if start.elapsed() >= timeout {
                        return Err(VirtualDeviceError::device_not_found(name, timeout_ms));
                    }
                    // Wait before polling again
                    std::thread::sleep(POLL_INTERVAL);
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Attempts to find and open a device by name (single poll).
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the device to find
    ///
    /// # Returns
    ///
    /// - `Ok(Some(capture))` if device found and opened
    /// - `Ok(None)` if device not found
    /// - `Err` if permission denied or other error
    fn try_find_device(name: &str) -> Result<Option<Self>, VirtualDeviceError> {
        let input_dir = Path::new("/dev/input");

        // Read directory entries
        let entries = match fs::read_dir(input_dir) {
            Ok(entries) => entries,
            Err(e) => {
                return Err(VirtualDeviceError::Io(e));
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue, // Skip entries we can't read
            };

            let path = entry.path();

            // Only consider event* devices
            match path.file_name().and_then(|n| n.to_str()) {
                Some(n) if n.starts_with("event") => {}
                _ => continue,
            }

            // Try to open the device
            let device = match Device::open(&path) {
                Ok(d) => d,
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("Permission denied") || err_str.contains("EACCES") {
                        return Err(VirtualDeviceError::evdev_permission_denied(
                            &path.to_string_lossy(),
                        ));
                    }
                    // Other errors (device busy, etc.) - skip and continue
                    continue;
                }
            };

            // Check if the name matches
            let device_name = device.name().unwrap_or("");
            if device_name == name {
                return Ok(Some(OutputCapture {
                    device,
                    name: name.to_string(),
                    device_path: path,
                }));
            }
        }

        // Device not found in this poll
        Ok(None)
    }

    /// Returns the name of the captured device.
    ///
    /// This is the name as reported by evdev, which should match the name
    /// used when creating the virtual keyboard.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let capture = OutputCapture::find_by_name("test-keyboard", timeout)?;
    /// assert_eq!(capture.name(), "test-keyboard");
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the device path (e.g., `/dev/input/event5`).
    ///
    /// Useful for debugging and logging which device was captured.
    #[must_use]
    pub fn device_path(&self) -> &Path {
        &self.device_path
    }

    /// Returns a reference to the underlying evdev device.
    ///
    /// This provides access to the raw device for advanced use cases
    /// or direct event reading.
    #[must_use]
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Returns a mutable reference to the underlying evdev device.
    ///
    /// This provides access to the raw device for event reading.
    pub fn device_mut(&mut self) -> &mut Device {
        &mut self.device
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poll_interval_is_reasonable() {
        // Poll interval should be between 10ms and 200ms
        assert!(POLL_INTERVAL >= Duration::from_millis(10));
        assert!(POLL_INTERVAL <= Duration::from_millis(200));
    }

    #[test]
    fn test_find_by_name_nonexistent_device() {
        // This should timeout quickly since the device doesn't exist
        let result =
            OutputCapture::find_by_name("nonexistent-device-12345", Duration::from_millis(100));

        assert!(result.is_err());

        match result {
            Err(VirtualDeviceError::NotFound { name, timeout_ms }) => {
                assert_eq!(name, "nonexistent-device-12345");
                assert!(timeout_ms >= 100);
            }
            Err(VirtualDeviceError::PermissionDenied { .. }) => {
                // Also acceptable if we don't have permission to read /dev/input
            }
            Err(e) => panic!("Unexpected error type: {:?}", e),
            Ok(_) => panic!("Should not have found nonexistent device"),
        }
    }

    #[test]
    fn test_find_by_name_timeout() {
        // Test that timeout works correctly
        let start = Instant::now();
        let timeout = Duration::from_millis(150);

        let result = OutputCapture::find_by_name("nonexistent-timeout-test", timeout);

        let elapsed = start.elapsed();

        // Should have waited approximately the timeout duration
        // Allow some tolerance for scheduling delays
        assert!(
            result.is_err(),
            "Should fail for nonexistent device: {:?}",
            result
        );

        // Check that we actually waited (at least 80% of timeout)
        // but didn't wait too long (timeout + reasonable overhead)
        match result {
            Err(VirtualDeviceError::NotFound { .. }) => {
                assert!(
                    elapsed >= Duration::from_millis(100),
                    "Should have waited near timeout: {:?}",
                    elapsed
                );
                assert!(
                    elapsed < Duration::from_millis(500),
                    "Should not wait too long: {:?}",
                    elapsed
                );
            }
            Err(VirtualDeviceError::PermissionDenied { .. }) => {
                // Permission denied is immediate, no timeout waiting
            }
            _ => {}
        }
    }

    /// Test finding a real virtual device created by VirtualKeyboard
    /// Note: Marked #[ignore] because it requires uinput access
    #[test]
    #[ignore = "requires uinput access - run with: sudo cargo test -p keyrx_daemon --features linux test_find_virtual_keyboard_device -- --ignored"]
    fn test_find_virtual_keyboard_device() {
        use crate::test_utils::VirtualKeyboard;

        // Create a virtual keyboard
        let keyboard = VirtualKeyboard::create("output-capture-test")
            .expect("Failed to create virtual keyboard");

        let device_name = keyboard.name().to_string();

        // Give the device a moment to be registered in the kernel
        std::thread::sleep(Duration::from_millis(100));

        // Try to find it with OutputCapture
        let capture = OutputCapture::find_by_name(&device_name, Duration::from_secs(2))
            .expect("Failed to find virtual keyboard device");

        // Verify the device was found correctly
        assert_eq!(capture.name(), device_name);
        assert!(capture
            .device_path()
            .to_string_lossy()
            .starts_with("/dev/input/event"));
    }

    /// Test that OutputCapture can find a device that appears after a delay
    /// Note: Marked #[ignore] because it requires uinput access
    #[test]
    #[ignore = "requires uinput access - run with: sudo cargo test -p keyrx_daemon --features linux test_find_device_with_delay -- --ignored"]
    fn test_find_device_with_delay() {
        use crate::test_utils::VirtualKeyboard;
        use std::thread;

        let device_name = format!(
            "delayed-device-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        );

        let name_clone = device_name.clone();

        // Start searching before the device exists
        let search_handle =
            thread::spawn(move || OutputCapture::find_by_name(&name_clone, Duration::from_secs(5)));

        // Wait a bit, then create the device
        thread::sleep(Duration::from_millis(200));

        let keyboard =
            VirtualKeyboard::create(&device_name).expect("Failed to create virtual keyboard");

        // Wait for the search to complete
        let result = search_handle.join().expect("Search thread panicked");
        let capture = result.expect("Failed to find delayed device");

        // Verify
        assert_eq!(capture.name(), keyboard.name());
    }

    /// Test that OutputCapture returns NotFound when device doesn't exist
    #[test]
    fn test_not_found_error_contains_details() {
        let result = OutputCapture::find_by_name(
            "unique-nonexistent-device-xyz789",
            Duration::from_millis(50),
        );

        match result {
            Err(VirtualDeviceError::NotFound { name, timeout_ms }) => {
                assert!(name.contains("xyz789"));
                assert!(timeout_ms >= 50);
            }
            Err(VirtualDeviceError::PermissionDenied { .. }) => {
                // Also valid if we can't read /dev/input
            }
            Err(e) => panic!("Unexpected error type: {:?}", e),
            Ok(_) => panic!("Should not succeed for nonexistent device"),
        }
    }
}
