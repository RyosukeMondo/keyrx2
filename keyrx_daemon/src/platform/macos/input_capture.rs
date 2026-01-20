//! macOS keyboard input capture using rdev.
//!
//! This module provides keyboard event capture using the rdev crate,
//! which wraps the macOS Accessibility API.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use crossbeam_channel::{Receiver, Sender};
use keyrx_core::runtime::KeyEvent;

use crate::platform::{DeviceError, InputDevice};
use super::keycode_map::rdev_key_to_keyrx;

/// macOS keyboard input capture.
///
/// Captures keyboard events using rdev::listen on a background thread.
pub struct MacosInputCapture {
    receiver: Receiver<KeyEvent>,
    _listen_thread: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl MacosInputCapture {
    /// Creates a new input capture instance.
    ///
    /// Spawns a background thread that listens for keyboard events using
    /// rdev::listen and forwards them to the channel.
    ///
    /// # Arguments
    ///
    /// * `receiver` - Channel receiver for keyboard events
    /// * `sender` - Channel sender for forwarding events from rdev thread
    pub fn new(receiver: Receiver<KeyEvent>, sender: Sender<KeyEvent>) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        // Spawn rdev::listen thread
        let listen_thread = thread::spawn(move || {
            log::info!("Starting rdev::listen thread");

            let callback = move |event: rdev::Event| {
                // Check if still running
                if !running_clone.load(Ordering::Relaxed) {
                    return;
                }

                // Convert rdev event to KeyEvent
                match event.event_type {
                    rdev::EventType::KeyPress(key) => {
                        if let Some(keycode) = rdev_key_to_keyrx(key) {
                            let timestamp_us = get_timestamp_us();
                            let key_event = KeyEvent::press(keycode)
                                .with_timestamp(timestamp_us);

                            // Send event through channel
                            if let Err(e) = sender.send(key_event) {
                                log::error!("Failed to send key press event: {}", e);
                            }
                        } else {
                            log::trace!("Unmapped key press: {:?}", key);
                        }
                    }
                    rdev::EventType::KeyRelease(key) => {
                        if let Some(keycode) = rdev_key_to_keyrx(key) {
                            let timestamp_us = get_timestamp_us();
                            let key_event = KeyEvent::release(keycode)
                                .with_timestamp(timestamp_us);

                            // Send event through channel
                            if let Err(e) = sender.send(key_event) {
                                log::error!("Failed to send key release event: {}", e);
                            }
                        } else {
                            log::trace!("Unmapped key release: {:?}", key);
                        }
                    }
                    _ => {
                        // Ignore mouse and other non-keyboard events
                    }
                }
            };

            // Start listening (blocking call)
            if let Err(e) = rdev::listen(callback) {
                log::error!("rdev::listen failed: {:?}", e);
            }

            log::info!("rdev::listen thread terminated");
        });

        Self {
            receiver,
            _listen_thread: Some(listen_thread),
            running,
        }
    }
}

/// Gets current timestamp in microseconds since UNIX epoch.
fn get_timestamp_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}

impl InputDevice for MacosInputCapture {
    fn next_event(&mut self) -> Result<KeyEvent, DeviceError> {
        // Blocking receive from channel - this is the main event loop
        self.receiver
            .recv()
            .map_err(|e| {
                log::error!("Channel receive error: {}", e);
                DeviceError::Io(std::io::Error::other("Input channel disconnected"))
            })
    }

    fn grab(&mut self) -> Result<(), DeviceError> {
        // macOS doesn't have an explicit grab mechanism like Linux's EVIOCGRAB.
        // Event capture via Accessibility API is already "exclusive" in the sense
        // that we control whether to propagate events or suppress them.
        Ok(())
    }

    fn release(&mut self) -> Result<(), DeviceError> {
        // No explicit release needed for macOS
        Ok(())
    }
}

impl Drop for MacosInputCapture {
    fn drop(&mut self) {
        log::info!("Shutting down MacosInputCapture");

        // Signal the listen thread to stop
        self.running.store(false, Ordering::Relaxed);

        // Note: We cannot safely stop rdev::listen thread as it's a blocking
        // operation without a built-in cancellation mechanism. The thread will
        // terminate when the process exits.
        log::debug!("MacosInputCapture cleanup complete");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::unbounded;
    use keyrx_core::config::keys::KeyCode;
    use std::time::Duration;

    #[test]
    fn test_macos_input_capture_creation() {
        // Create channel
        let (sender, receiver) = unbounded();

        // Create input capture (will spawn rdev::listen thread)
        let _capture = MacosInputCapture::new(receiver, sender);

        // Just verify it can be created without panicking
        // The rdev::listen thread will fail without Accessibility permission,
        // but that's expected in test environment
    }

    #[test]
    fn test_channel_communication() {
        // Create channel
        let (sender, receiver) = unbounded();

        // Send a test event directly through the channel
        let test_event = KeyEvent::press(KeyCode::A).with_timestamp(12345);
        sender.send(test_event.clone()).unwrap();

        // Create input capture
        let mut capture = MacosInputCapture::new(receiver.clone(), sender);

        // Try to receive the event we sent
        let received = capture.next_event().unwrap();
        assert_eq!(received.keycode(), KeyCode::A);
        assert_eq!(received.timestamp_us(), 12345);
    }

    #[test]
    fn test_grab_and_release_no_op() {
        // Create channel
        let (sender, receiver) = unbounded();
        let mut capture = MacosInputCapture::new(receiver, sender);

        // These should be no-ops on macOS
        assert!(capture.grab().is_ok());
        assert!(capture.release().is_ok());
    }

    #[test]
    fn test_get_timestamp_us() {
        let timestamp1 = get_timestamp_us();
        std::thread::sleep(Duration::from_millis(1));
        let timestamp2 = get_timestamp_us();

        // Timestamp should increase
        assert!(timestamp2 > timestamp1);

        // Timestamp should be reasonable (not zero, not too large)
        assert!(timestamp1 > 0);
        assert!(timestamp1 < u64::MAX / 2);
    }

    #[test]
    fn test_multiple_events() {
        // Create channel
        let (sender, receiver) = unbounded();

        // Send multiple events
        sender.send(KeyEvent::press(KeyCode::A)).unwrap();
        sender.send(KeyEvent::release(KeyCode::A)).unwrap();
        sender.send(KeyEvent::press(KeyCode::B)).unwrap();

        // Create input capture
        let mut capture = MacosInputCapture::new(receiver.clone(), sender);

        // Receive events
        let event1 = capture.next_event().unwrap();
        assert_eq!(event1.keycode(), KeyCode::A);
        assert!(event1.is_press());

        let event2 = capture.next_event().unwrap();
        assert_eq!(event2.keycode(), KeyCode::A);
        assert!(event2.is_release());

        let event3 = capture.next_event().unwrap();
        assert_eq!(event3.keycode(), KeyCode::B);
        assert!(event3.is_press());
    }

    #[test]
    fn test_running_flag() {
        // Create channel
        let (sender, receiver) = unbounded();
        let capture = MacosInputCapture::new(receiver, sender);

        // Should be running initially
        assert!(capture.running.load(Ordering::Relaxed));

        // After drop, should be false
        drop(capture);
        // Note: We can't test this after drop since capture is moved
    }
}
