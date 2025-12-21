//! Configuration loading and validation module
//!
//! This module handles loading and validating .krx binary configuration files
//! using rkyv for zero-copy deserialization.

use core::fmt;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

/// Version information for binary compatibility checking.
///
/// Uses semantic versioning with major.minor.patch components.
/// All fields are u8 to keep the size small (3 bytes total).
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct Version {
    /// Major version number (incompatible API changes)
    pub major: u8,
    /// Minor version number (backwards-compatible functionality)
    pub minor: u8,
    /// Patch version number (backwards-compatible bug fixes)
    pub patch: u8,
}

impl Version {
    /// Returns the current version of the configuration format.
    ///
    /// # Returns
    /// Version 1.0.0 - the initial stable version
    pub fn current() -> Self {
        Self {
            major: 1,
            minor: 0,
            patch: 0,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
