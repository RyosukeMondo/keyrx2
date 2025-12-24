//! KeyRx daemon library.
//!
//! This library provides platform abstraction and event processing for the KeyRx daemon.

// test_utils must come first to make macros available to other modules
#[cfg(feature = "linux")]
pub mod test_utils;

pub mod config_loader;
#[cfg(any(feature = "linux", feature = "windows"))]
pub mod daemon;
pub mod device_manager;
pub mod platform;
pub mod processor;

#[cfg(feature = "web")]
pub mod web;
