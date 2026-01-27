//! Linux-specific domain models for Platform domain

#![cfg(target_os = "linux")]

pub mod aggregates;
pub mod services;
pub mod value_objects;

// Re-export key types
pub use aggregates::{EvdevDeviceAggregate, UinputDeviceAggregate};
pub use services::{EvdevCaptureService, UinputInjectionService};
pub use value_objects::{DeviceFdVO, EventCodeVO};
