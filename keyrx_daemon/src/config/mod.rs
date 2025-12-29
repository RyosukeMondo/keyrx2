//! Configuration management module
//!
//! This module provides components for managing device metadata,
//! profiles, layouts, and configuration generation.

pub mod device_registry;

pub use device_registry::{DeviceEntry, DeviceRegistry, DeviceScope, RegistryError};
