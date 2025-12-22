//! KeyRx daemon library.
//!
//! This library provides platform abstraction and event processing for the KeyRx daemon.

pub mod config_loader;
pub mod platform;

#[cfg(feature = "web")]
pub mod web;
