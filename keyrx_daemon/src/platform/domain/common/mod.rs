//! Common domain models for Platform domain
//!
//! This module contains platform-agnostic domain models that are shared
//! across Linux and Windows implementations.

pub mod aggregates;
pub mod repositories;
pub mod value_objects;

// Re-export key types
pub use aggregates::PlatformDeviceAggregate;
pub use repositories::PlatformDeviceRepository;
pub use value_objects::{DeviceHandleVO, DevicePathVO};
