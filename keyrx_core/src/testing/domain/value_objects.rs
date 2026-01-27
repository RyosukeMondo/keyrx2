//! Domain value objects for Testing domain
//!
//! Value objects are immutable and defined by their attributes, not identity.

use super::DomainError;

/// Timestamp value object for deterministic testing
///
/// Wraps u64 microseconds with deterministic time operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimestampVO(u64);

impl TimestampVO {
    /// Creates a new Timestamp value object
    pub fn new(microseconds: u64) -> Self {
        Self(microseconds)
    }

    /// Gets the timestamp in microseconds
    pub fn as_microseconds(&self) -> u64 {
        self.0
    }

    /// Gets the timestamp in milliseconds
    pub fn as_milliseconds(&self) -> u64 {
        self.0 / 1000
    }

    /// Gets the timestamp in seconds
    pub fn as_seconds(&self) -> u64 {
        self.0 / 1_000_000
    }

    /// Advances the timestamp by a delta
    pub fn advance(&self, delta_us: u64) -> Self {
        Self(self.0.saturating_add(delta_us))
    }

    /// Calculates the duration between two timestamps
    pub fn duration_since(&self, other: TimestampVO) -> Option<u64> {
        if self.0 >= other.0 {
            Some(self.0 - other.0)
        } else {
            None
        }
    }

    /// Checks if this timestamp is after another
    pub fn is_after(&self, other: TimestampVO) -> bool {
        self.0 > other.0
    }

    /// Checks if this timestamp is before another
    pub fn is_before(&self, other: TimestampVO) -> bool {
        self.0 < other.0
    }

    /// Creates a zero timestamp (epoch)
    pub fn zero() -> Self {
        Self(0)
    }
}

impl Default for TimestampVO {
    fn default() -> Self {
        Self::zero()
    }
}

impl From<u64> for TimestampVO {
    fn from(microseconds: u64) -> Self {
        Self(microseconds)
    }
}

/// Seed value object for deterministic random number generation
///
/// Wraps a u64 seed value for reproducible test execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeedVO(u64);

impl SeedVO {
    /// Creates a new Seed value object
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Gets the inner seed value
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Creates a seed from a string (hash-based)
    pub fn from_string(s: &str) -> Self {
        // Simple hash function for deterministic seed from string
        let mut hash = 0u64;
        for byte in s.as_bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(*byte as u64);
        }
        Self(hash)
    }

    /// Derives a new seed from this seed
    ///
    /// Useful for creating independent RNG streams in nested contexts.
    pub fn derive(&self, index: u64) -> Self {
        // Mix the seed with the index using a simple mixing function
        let mixed = self.0.wrapping_mul(6364136223846793005u64).wrapping_add(index);
        Self(mixed)
    }

    /// Validates this seed for use
    pub fn validate(&self) -> Result<(), DomainError> {
        // All seeds are valid, but we can add constraints if needed
        Ok(())
    }

    /// Creates a default seed (deterministic)
    pub fn default_seed() -> Self {
        Self(42)
    }
}

impl Default for SeedVO {
    fn default() -> Self {
        Self::default_seed()
    }
}

impl From<u64> for SeedVO {
    fn from(seed: u64) -> Self {
        Self(seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_vo_creation() {
        let ts = TimestampVO::new(1_000_000);
        assert_eq!(ts.as_microseconds(), 1_000_000);
        assert_eq!(ts.as_milliseconds(), 1_000);
        assert_eq!(ts.as_seconds(), 1);
    }

    #[test]
    fn test_timestamp_vo_advance() {
        let ts1 = TimestampVO::new(1000);
        let ts2 = ts1.advance(500);

        assert_eq!(ts2.as_microseconds(), 1500);
        assert!(ts2.is_after(ts1));
        assert!(ts1.is_before(ts2));
    }

    #[test]
    fn test_timestamp_vo_duration() {
        let ts1 = TimestampVO::new(1000);
        let ts2 = TimestampVO::new(1500);

        assert_eq!(ts2.duration_since(ts1), Some(500));
        assert_eq!(ts1.duration_since(ts2), None);
    }

    #[test]
    fn test_timestamp_vo_ordering() {
        let ts1 = TimestampVO::new(1000);
        let ts2 = TimestampVO::new(2000);
        let ts3 = TimestampVO::new(1000);

        assert!(ts1 < ts2);
        assert!(ts2 > ts1);
        assert_eq!(ts1, ts3);
    }

    #[test]
    fn test_timestamp_vo_zero() {
        let zero = TimestampVO::zero();
        assert_eq!(zero.as_microseconds(), 0);

        let default = TimestampVO::default();
        assert_eq!(default, zero);
    }

    #[test]
    fn test_seed_vo_creation() {
        let seed = SeedVO::new(12345);
        assert_eq!(seed.value(), 12345);
    }

    #[test]
    fn test_seed_vo_from_string() {
        let seed1 = SeedVO::from_string("test");
        let seed2 = SeedVO::from_string("test");
        let seed3 = SeedVO::from_string("different");

        // Same string produces same seed
        assert_eq!(seed1, seed2);

        // Different string produces different seed
        assert_ne!(seed1, seed3);
    }

    #[test]
    fn test_seed_vo_derive() {
        let seed = SeedVO::new(12345);
        let derived1 = seed.derive(0);
        let derived2 = seed.derive(1);

        // Derived seeds should be different from each other
        assert_ne!(derived1, derived2);

        // Derived seeds should be different from parent
        assert_ne!(derived1, seed);
        assert_ne!(derived2, seed);

        // Same derivation should produce same result
        let derived1_again = seed.derive(0);
        assert_eq!(derived1, derived1_again);
    }

    #[test]
    fn test_seed_vo_validate() {
        let seed = SeedVO::new(12345);
        assert!(seed.validate().is_ok());

        let zero_seed = SeedVO::new(0);
        assert!(zero_seed.validate().is_ok());
    }

    #[test]
    fn test_seed_vo_default() {
        let default = SeedVO::default();
        assert_eq!(default, SeedVO::default_seed());
        assert_eq!(default.value(), 42);
    }

    #[test]
    fn test_timestamp_vo_saturating_add() {
        let ts = TimestampVO::new(u64::MAX - 100);
        let advanced = ts.advance(200);

        // Should saturate at u64::MAX
        assert_eq!(advanced.as_microseconds(), u64::MAX);
    }

    #[test]
    fn test_seed_vo_equality() {
        let seed1 = SeedVO::new(12345);
        let seed2 = SeedVO::new(12345);
        let seed3 = SeedVO::new(54321);

        assert_eq!(seed1, seed2);
        assert_ne!(seed1, seed3);
    }
}
