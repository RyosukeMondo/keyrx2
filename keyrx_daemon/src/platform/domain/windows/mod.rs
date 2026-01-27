//! Windows-specific domain models for Platform domain

#![cfg(target_os = "windows")]

pub mod aggregates;
pub mod services;
pub mod value_objects;

// Re-export key types
pub use aggregates::{HookCallbackAggregate, RawInputDeviceAggregate};
pub use services::{LowLevelHookService, SendInputService};
pub use value_objects::{ScanCodeVO, VirtualKeyCodeVO};
