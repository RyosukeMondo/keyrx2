//! Parser state shared across Rhai custom functions.

use crate::config::{BaseKeyMapping, Condition, DeviceConfig};
use alloc::vec::Vec;

/// Parser state shared across Rhai custom functions.
#[derive(Debug, Clone, Default)]
pub struct ParserState {
    /// Collected device configurations
    pub devices: Vec<DeviceConfig>,
    /// Current device being configured (between device_start and device_end)
    pub current_device: Option<DeviceConfig>,
    /// Stack of (Condition, mappings) pairs being collected for conditional blocks
    /// When non-empty, map() adds to the top of this stack instead of current_device
    pub conditional_stack: Vec<(Condition, Vec<BaseKeyMapping>)>,
}

impl ParserState {
    /// Create a new empty parser state.
    pub fn new() -> Self {
        Self::default()
    }
}
