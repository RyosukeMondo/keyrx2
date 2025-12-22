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
use std::os::fd::BorrowedFd;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use evdev::{Device, InputEventKind};
use nix::poll::{poll, PollFd, PollFlags, PollTimeout};

use keyrx_core::runtime::event::KeyEvent;

use super::VirtualDeviceError;
use crate::platform::linux::evdev_to_keycode;

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

    /// Reads the next keyboard event with a timeout.
    ///
    /// This method uses non-blocking I/O with poll to wait for events.
    /// Only EV_KEY events are returned; other event types (EV_SYN, EV_MSC, etc.)
    /// are filtered out.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for an event
    ///
    /// # Returns
    ///
    /// - `Ok(Some(KeyEvent))` if a keyboard event was received
    /// - `Ok(None)` if the timeout expired without any keyboard events
    /// - `Err(VirtualDeviceError::Io)` on I/O errors
    ///
    /// # Event Filtering
    ///
    /// - Key press (value=1) → `KeyEvent::Press`
    /// - Key release (value=0) → `KeyEvent::Release`
    /// - Key repeat (value=2) → Ignored (continues waiting)
    /// - Non-key events → Ignored (continues waiting)
    /// - Unknown keys → Ignored (continues waiting)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use keyrx_daemon::test_utils::OutputCapture;
    /// use std::time::Duration;
    ///
    /// let mut capture = OutputCapture::find_by_name("keyrx Virtual Keyboard", timeout)?;
    ///
    /// match capture.next_event(Duration::from_millis(100))? {
    ///     Some(event) => println!("Got event: {:?}", event),
    ///     None => println!("Timeout - no events received"),
    /// }
    /// ```
    pub fn next_event(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<KeyEvent>, VirtualDeviceError> {
        let start = Instant::now();

        loop {
            // Calculate remaining timeout
            let elapsed = start.elapsed();
            if elapsed >= timeout {
                return Ok(None);
            }
            let remaining = timeout - elapsed;
            let remaining_ms = remaining.as_millis().min(u16::MAX as u128) as u16;

            // Get a borrowed fd for polling
            // SAFETY: The raw fd is valid for the lifetime of the loop iteration since
            // we hold &mut self, ensuring the device stays alive
            let borrowed_fd = unsafe { BorrowedFd::borrow_raw(self.device.as_raw_fd()) };

            // Poll for readable events
            let mut poll_fds = [PollFd::new(borrowed_fd, PollFlags::POLLIN)];
            let poll_timeout = PollTimeout::from(remaining_ms);
            match poll(&mut poll_fds, poll_timeout) {
                Ok(0) => {
                    // Timeout, no events
                    return Ok(None);
                }
                Ok(_) => {
                    // Events available, try to read them
                    if let Some(event) = self.try_read_key_event()? {
                        return Ok(Some(event));
                    }
                    // Got non-key events, continue polling
                }
                Err(nix::errno::Errno::EINTR) => {
                    // Interrupted by signal, continue polling
                    continue;
                }
                Err(e) => {
                    return Err(VirtualDeviceError::Io(std::io::Error::other(format!(
                        "poll failed: {}",
                        e
                    ))));
                }
            }
        }
    }

    /// Tries to read a single key event from the device.
    ///
    /// Returns `Ok(Some(event))` if a key event was read, `Ok(None)` if only
    /// non-key events were available, or `Err` on I/O error.
    fn try_read_key_event(&mut self) -> Result<Option<KeyEvent>, VirtualDeviceError> {
        // Fetch available events (non-blocking after poll)
        let events = match self.device.fetch_events() {
            Ok(events) => events,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                return Ok(None);
            }
            Err(e) => {
                return Err(VirtualDeviceError::Io(e));
            }
        };

        for event in events {
            // Only process EV_KEY events
            if let InputEventKind::Key(key) = event.kind() {
                let value = event.value();

                match value {
                    1 => {
                        // Key press
                        if let Some(keycode) = evdev_to_keycode(key.code()) {
                            return Ok(Some(KeyEvent::Press(keycode)));
                        }
                        // Unknown key - continue to next event
                    }
                    0 => {
                        // Key release
                        if let Some(keycode) = evdev_to_keycode(key.code()) {
                            return Ok(Some(KeyEvent::Release(keycode)));
                        }
                        // Unknown key - continue to next event
                    }
                    2 => {
                        // Key repeat - ignore
                    }
                    _ => {
                        // Unknown event value - ignore
                    }
                }
            }
            // Non-key events (EV_SYN, EV_MSC, etc.) are ignored
        }

        // No key events found in this batch
        Ok(None)
    }

    /// Collects keyboard events until the timeout expires.
    ///
    /// This method continues reading events until the specified timeout
    /// elapses with no new events. Events are returned in the order received.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Time to wait for additional events after receiving each event.
    ///   The timeout resets after each event, so this is effectively the
    ///   "idle timeout" for the collection.
    ///
    /// # Returns
    ///
    /// A vector of collected keyboard events (may be empty if timeout with no events).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use keyrx_daemon::test_utils::OutputCapture;
    /// use std::time::Duration;
    ///
    /// let mut capture = OutputCapture::find_by_name("test-keyboard", timeout)?;
    ///
    /// // Collect all events that arrive within 100ms gaps
    /// let events = capture.collect_events(Duration::from_millis(100))?;
    /// println!("Collected {} events", events.len());
    /// ```
    pub fn collect_events(
        &mut self,
        timeout: Duration,
    ) -> Result<Vec<KeyEvent>, VirtualDeviceError> {
        let mut events = Vec::new();

        loop {
            match self.next_event(timeout)? {
                Some(event) => {
                    events.push(event);
                    // Continue collecting with fresh timeout
                }
                None => {
                    // Timeout expired, return collected events
                    return Ok(events);
                }
            }
        }
    }

    /// Drains and discards all pending events from the device.
    ///
    /// This is useful before starting a test to ensure no stale events
    /// from previous operations affect the test results.
    ///
    /// # Returns
    ///
    /// The number of events that were drained (for debugging/logging).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use keyrx_daemon::test_utils::OutputCapture;
    ///
    /// let mut capture = OutputCapture::find_by_name("test-keyboard", timeout)?;
    ///
    /// // Clear any pending events before starting test
    /// let drained = capture.drain()?;
    /// if drained > 0 {
    ///     println!("Drained {} stale events", drained);
    /// }
    /// ```
    pub fn drain(&mut self) -> Result<usize, VirtualDeviceError> {
        let mut count = 0;

        loop {
            // Get a borrowed fd for polling
            // SAFETY: The raw fd is valid for the lifetime of the loop iteration since
            // we hold &mut self, ensuring the device stays alive
            let borrowed_fd = unsafe { BorrowedFd::borrow_raw(self.device.as_raw_fd()) };

            // Non-blocking poll with zero timeout
            let mut poll_fds = [PollFd::new(borrowed_fd, PollFlags::POLLIN)];
            // Use PollTimeout::ZERO for immediate return
            match poll(&mut poll_fds, PollTimeout::ZERO) {
                Ok(0) => {
                    // No more events pending
                    return Ok(count);
                }
                Ok(_) => {
                    // Events available, read and discard them
                    match self.device.fetch_events() {
                        Ok(events) => {
                            // Count all events (including non-key events)
                            count += events.count();
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            return Ok(count);
                        }
                        Err(e) => {
                            return Err(VirtualDeviceError::Io(e));
                        }
                    }
                }
                Err(nix::errno::Errno::EINTR) => {
                    continue;
                }
                Err(e) => {
                    return Err(VirtualDeviceError::Io(std::io::Error::other(format!(
                        "poll failed during drain: {}",
                        e
                    ))));
                }
            }
        }
    }
}

/// A captured keyboard event with its keycode.
///
/// This is a convenience type alias for the core `KeyEvent` type,
/// used for test assertions and comparisons.
#[allow(dead_code)] // Type alias for documentation/clarity
pub type CapturedEvent = KeyEvent;

#[cfg(test)]
mod tests {
    use super::*;
    use keyrx_core::config::KeyCode;

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

    /// Test next_event returns None on timeout with no events
    /// Note: Marked #[ignore] because it requires uinput access
    #[test]
    #[ignore = "requires uinput access - run with: sudo cargo test -p keyrx_daemon --features linux test_next_event_timeout -- --ignored"]
    fn test_next_event_timeout() {
        use crate::test_utils::VirtualKeyboard;

        // Create a virtual keyboard
        let keyboard = VirtualKeyboard::create("next-event-timeout-test")
            .expect("Failed to create virtual keyboard");

        let device_name = keyboard.name().to_string();

        // Give the device a moment to be registered
        std::thread::sleep(Duration::from_millis(100));

        // Find the device with OutputCapture
        let mut capture = OutputCapture::find_by_name(&device_name, Duration::from_secs(2))
            .expect("Failed to find device");

        // Drain any pending events first
        let _ = capture.drain();

        // Try to read with a short timeout - should return None
        let start = Instant::now();
        let result = capture
            .next_event(Duration::from_millis(50))
            .expect("next_event failed");
        let elapsed = start.elapsed();

        assert!(result.is_none(), "Should timeout with no events");
        assert!(
            elapsed >= Duration::from_millis(40),
            "Should wait near timeout: {:?}",
            elapsed
        );
        assert!(
            elapsed < Duration::from_millis(200),
            "Should not wait too long: {:?}",
            elapsed
        );
    }

    /// Test next_event captures injected key events
    /// Note: Marked #[ignore] because it requires uinput access
    #[test]
    #[ignore = "requires uinput access - run with: sudo cargo test -p keyrx_daemon --features linux test_next_event_captures_key -- --ignored"]
    fn test_next_event_captures_key() {
        use crate::test_utils::VirtualKeyboard;

        // Create a virtual keyboard
        let mut keyboard = VirtualKeyboard::create("next-event-capture-test")
            .expect("Failed to create virtual keyboard");

        let device_name = keyboard.name().to_string();

        // Give the device a moment to be registered
        std::thread::sleep(Duration::from_millis(100));

        // Find the device with OutputCapture
        let mut capture = OutputCapture::find_by_name(&device_name, Duration::from_secs(2))
            .expect("Failed to find device");

        // Drain any pending events
        let _ = capture.drain();

        // Inject a key press
        keyboard
            .inject(KeyEvent::Press(KeyCode::A))
            .expect("Failed to inject key");

        // Read the event
        let event = capture
            .next_event(Duration::from_millis(500))
            .expect("next_event failed")
            .expect("Should have received an event");

        assert_eq!(event, KeyEvent::Press(KeyCode::A));
    }

    /// Test collect_events gathers multiple events
    /// Note: Marked #[ignore] because it requires uinput access
    #[test]
    #[ignore = "requires uinput access - run with: sudo cargo test -p keyrx_daemon --features linux test_collect_events_multiple -- --ignored"]
    fn test_collect_events_multiple() {
        use crate::test_utils::VirtualKeyboard;

        // Create a virtual keyboard
        let mut keyboard = VirtualKeyboard::create("collect-events-test")
            .expect("Failed to create virtual keyboard");

        let device_name = keyboard.name().to_string();

        // Give the device a moment to be registered
        std::thread::sleep(Duration::from_millis(100));

        // Find the device with OutputCapture
        let mut capture = OutputCapture::find_by_name(&device_name, Duration::from_secs(2))
            .expect("Failed to find device");

        // Drain any pending events
        let _ = capture.drain();

        // Inject a key tap sequence (press + release)
        keyboard
            .inject(KeyEvent::Press(KeyCode::B))
            .expect("Failed to inject key press");
        keyboard
            .inject(KeyEvent::Release(KeyCode::B))
            .expect("Failed to inject key release");

        // Collect all events
        let events = capture
            .collect_events(Duration::from_millis(100))
            .expect("collect_events failed");

        assert_eq!(events.len(), 2, "Should collect 2 events");
        assert_eq!(events[0], KeyEvent::Press(KeyCode::B));
        assert_eq!(events[1], KeyEvent::Release(KeyCode::B));
    }

    /// Test collect_events returns empty vector on timeout
    /// Note: Marked #[ignore] because it requires uinput access
    #[test]
    #[ignore = "requires uinput access - run with: sudo cargo test -p keyrx_daemon --features linux test_collect_events_empty -- --ignored"]
    fn test_collect_events_empty() {
        use crate::test_utils::VirtualKeyboard;

        // Create a virtual keyboard
        let keyboard = VirtualKeyboard::create("collect-events-empty-test")
            .expect("Failed to create virtual keyboard");

        let device_name = keyboard.name().to_string();

        // Give the device a moment to be registered
        std::thread::sleep(Duration::from_millis(100));

        // Find the device with OutputCapture
        let mut capture = OutputCapture::find_by_name(&device_name, Duration::from_secs(2))
            .expect("Failed to find device");

        // Drain any pending events
        let _ = capture.drain();

        // Collect with no events - should return empty
        let events = capture
            .collect_events(Duration::from_millis(50))
            .expect("collect_events failed");

        assert!(events.is_empty(), "Should return empty vector on timeout");
    }

    /// Test drain clears pending events
    /// Note: Marked #[ignore] because it requires uinput access
    #[test]
    #[ignore = "requires uinput access - run with: sudo cargo test -p keyrx_daemon --features linux test_drain_clears_events -- --ignored"]
    fn test_drain_clears_events() {
        use crate::test_utils::VirtualKeyboard;

        // Create a virtual keyboard
        let mut keyboard = VirtualKeyboard::create("drain-events-test")
            .expect("Failed to create virtual keyboard");

        let device_name = keyboard.name().to_string();

        // Give the device a moment to be registered
        std::thread::sleep(Duration::from_millis(100));

        // Find the device with OutputCapture
        let mut capture = OutputCapture::find_by_name(&device_name, Duration::from_secs(2))
            .expect("Failed to find device");

        // Inject some events
        keyboard
            .inject(KeyEvent::Press(KeyCode::C))
            .expect("Failed to inject key press");
        keyboard
            .inject(KeyEvent::Release(KeyCode::C))
            .expect("Failed to inject key release");

        // Give events time to be received
        std::thread::sleep(Duration::from_millis(50));

        // Drain should clear the events
        let drained = capture.drain().expect("drain failed");
        assert!(drained > 0, "Should have drained some events");

        // Now next_event should timeout
        let result = capture
            .next_event(Duration::from_millis(50))
            .expect("next_event failed");
        assert!(result.is_none(), "Should have no events after drain");
    }

    /// Test drain returns 0 when no events pending
    /// Note: Marked #[ignore] because it requires uinput access
    #[test]
    #[ignore = "requires uinput access - run with: sudo cargo test -p keyrx_daemon --features linux test_drain_no_events -- --ignored"]
    fn test_drain_no_events() {
        use crate::test_utils::VirtualKeyboard;

        // Create a virtual keyboard
        let keyboard = VirtualKeyboard::create("drain-no-events-test")
            .expect("Failed to create virtual keyboard");

        let device_name = keyboard.name().to_string();

        // Give the device a moment to be registered
        std::thread::sleep(Duration::from_millis(100));

        // Find the device with OutputCapture
        let mut capture = OutputCapture::find_by_name(&device_name, Duration::from_secs(2))
            .expect("Failed to find device");

        // Drain twice to ensure no events
        let _ = capture.drain();
        let drained = capture.drain().expect("drain failed");

        assert_eq!(drained, 0, "Should return 0 when no events pending");
    }

    /// Test that CapturedEvent type alias works correctly
    #[test]
    fn test_captured_event_type_alias() {
        // This just verifies the type alias compiles and works
        let event: CapturedEvent = KeyEvent::Press(KeyCode::D);
        assert_eq!(event, KeyEvent::Press(KeyCode::D));
    }
}
