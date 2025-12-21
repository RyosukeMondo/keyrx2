//! Configuration submodules re-exported for convenience

pub mod conditions;
pub mod keys;
pub mod mappings;
pub mod types;

// Re-export core types
pub use conditions::{Condition, ConditionItem};
pub use keys::KeyCode;
pub use mappings::{BaseKeyMapping, ConfigRoot, DeviceConfig, DeviceIdentifier, KeyMapping};
pub use types::{Metadata, Version};
