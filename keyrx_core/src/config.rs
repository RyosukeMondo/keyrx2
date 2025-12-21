//! Configuration data structures for KeyRx
//!
//! This module defines all configuration types using rkyv for zero-copy deserialization.
//! All types use #[repr(C)] for stable binary layout.

use core::fmt;
use rkyv::{Archive, Deserialize, Serialize};

/// Version information for binary compatibility checking
///
/// Uses semantic versioning with major.minor.patch format.
/// All fields are u8 to keep the struct compact.
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl Version {
    /// Returns the current version (1.0.0)
    pub const fn current() -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::string::ToString;

    #[test]
    fn test_version_current() {
        let version = Version::current();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
    }

    #[test]
    fn test_version_display() {
        let version = Version::current();
        assert_eq!(version.to_string(), "1.0.0");
    }
}
