//! Tap-hold state machine implementation.
//!
//! This module provides the core tap-hold functionality where a key can act as
//! one key when tapped (quick press and release) and a modifier when held
//! (pressed beyond a threshold).
//!
//! # State Machine
//!
//! ```text
//!                    Press
//!     ┌─────────────────────────────────────┐
//!     │                                     ▼
//!  ┌──────┐    ┌─────────┐  timeout    ┌────────┐
//!  │ Idle │───▶│ Pending │────────────▶│  Hold  │
//!  └──────┘    └─────────┘             └────────┘
//!     ▲             │                       │
//!     │   quick     │    other key          │
//!     │   release   │    pressed            │
//!     │   (tap)     │  (permissive hold)    │
//!     │             ▼                       │
//!     │         emit tap                    │
//!     │         key event                   │
//!     │                                     │
//!     └─────────────────────────────────────┘
//!                   Release
//! ```
//!
//! # Example
//!
//! ```rust
//! use keyrx_core::runtime::tap_hold::{TapHoldPhase, TapHoldState, TapHoldConfig};
//! use keyrx_core::config::KeyCode;
//!
//! // Configure CapsLock as tap=Escape, hold=Ctrl (modifier 0)
//! let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
//!
//! // Create initial state
//! let state = TapHoldState::new(KeyCode::CapsLock, config);
//!
//! assert_eq!(state.phase(), TapHoldPhase::Idle);
//! assert_eq!(state.key(), KeyCode::CapsLock);
//! ```

use arrayvec::ArrayVec;

use crate::config::KeyCode;

/// Phase of the tap-hold state machine.
///
/// # Phases
///
/// - `Idle`: No key activity, waiting for press
/// - `Pending`: Key pressed, waiting to determine tap vs hold
/// - `Hold`: Key held past threshold, modifier active
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TapHoldPhase {
    /// No activity, waiting for key press
    #[default]
    Idle,
    /// Key pressed, waiting for release or timeout
    Pending,
    /// Key held, modifier is active
    Hold,
}

impl TapHoldPhase {
    /// Returns true if the phase is Idle.
    pub const fn is_idle(&self) -> bool {
        matches!(self, TapHoldPhase::Idle)
    }

    /// Returns true if the phase is Pending.
    pub const fn is_pending(&self) -> bool {
        matches!(self, TapHoldPhase::Pending)
    }

    /// Returns true if the phase is Hold.
    pub const fn is_hold(&self) -> bool {
        matches!(self, TapHoldPhase::Hold)
    }
}

/// Configuration for a tap-hold key.
///
/// Contains the behavior settings for a single tap-hold key:
/// - What key to emit on tap
/// - What modifier to activate on hold
/// - Threshold time in microseconds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TapHoldConfig {
    /// Key to emit when tapped (quick press and release)
    tap_key: KeyCode,
    /// Modifier ID to activate when held (0-254)
    hold_modifier: u8,
    /// Threshold time in microseconds (tap vs hold boundary)
    threshold_us: u64,
}

impl TapHoldConfig {
    /// Creates a new tap-hold configuration.
    ///
    /// # Arguments
    ///
    /// * `tap_key` - Key to emit on tap
    /// * `hold_modifier` - Modifier ID to activate on hold (0-254)
    /// * `threshold_us` - Time in microseconds to distinguish tap from hold
    ///
    /// # Example
    ///
    /// ```rust
    /// use keyrx_core::runtime::tap_hold::TapHoldConfig;
    /// use keyrx_core::config::KeyCode;
    ///
    /// // CapsLock: tap=Escape, hold=Ctrl (200ms threshold)
    /// let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
    ///
    /// assert_eq!(config.tap_key(), KeyCode::Escape);
    /// assert_eq!(config.hold_modifier(), 0);
    /// assert_eq!(config.threshold_us(), 200_000);
    /// ```
    pub const fn new(tap_key: KeyCode, hold_modifier: u8, threshold_us: u64) -> Self {
        Self {
            tap_key,
            hold_modifier,
            threshold_us,
        }
    }

    /// Creates a config from milliseconds threshold.
    ///
    /// Convenience constructor for common millisecond-based thresholds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use keyrx_core::runtime::tap_hold::TapHoldConfig;
    /// use keyrx_core::config::KeyCode;
    ///
    /// let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    /// assert_eq!(config.threshold_us(), 200_000);
    /// ```
    pub const fn from_ms(tap_key: KeyCode, hold_modifier: u8, threshold_ms: u16) -> Self {
        Self::new(tap_key, hold_modifier, threshold_ms as u64 * 1000)
    }

    /// Returns the tap key.
    pub const fn tap_key(&self) -> KeyCode {
        self.tap_key
    }

    /// Returns the hold modifier ID.
    pub const fn hold_modifier(&self) -> u8 {
        self.hold_modifier
    }

    /// Returns the threshold in microseconds.
    pub const fn threshold_us(&self) -> u64 {
        self.threshold_us
    }
}

/// State for a single tap-hold key.
///
/// Tracks the current phase, timing, and configuration for one tap-hold key.
/// Multiple instances can be tracked simultaneously via `PendingKeyRegistry`.
///
/// # Example
///
/// ```rust
/// use keyrx_core::runtime::tap_hold::{TapHoldPhase, TapHoldState, TapHoldConfig};
/// use keyrx_core::config::KeyCode;
///
/// let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
/// let mut state = TapHoldState::new(KeyCode::CapsLock, config);
///
/// // Initially idle
/// assert!(state.phase().is_idle());
///
/// // Transition to pending on press
/// state.transition_to_pending(1000);
/// assert!(state.phase().is_pending());
/// assert_eq!(state.press_time(), 1000);
///
/// // Check if threshold exceeded
/// assert!(!state.is_threshold_exceeded(100_000)); // 99ms < 200ms
/// assert!(state.is_threshold_exceeded(300_000));  // 299ms > 200ms
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TapHoldState {
    /// The physical key that triggers this tap-hold
    key: KeyCode,
    /// Current state machine phase
    phase: TapHoldPhase,
    /// Configuration for tap/hold behavior
    config: TapHoldConfig,
    /// Timestamp when key was pressed (microseconds)
    press_time: u64,
}

impl TapHoldState {
    /// Creates a new tap-hold state in Idle phase.
    ///
    /// # Arguments
    ///
    /// * `key` - The physical key that triggers this tap-hold
    /// * `config` - Configuration for tap/hold behavior
    pub const fn new(key: KeyCode, config: TapHoldConfig) -> Self {
        Self {
            key,
            phase: TapHoldPhase::Idle,
            config,
            press_time: 0,
        }
    }

    /// Returns the physical key.
    pub const fn key(&self) -> KeyCode {
        self.key
    }

    /// Returns the current phase.
    pub const fn phase(&self) -> TapHoldPhase {
        self.phase
    }

    /// Returns the configuration.
    pub const fn config(&self) -> &TapHoldConfig {
        &self.config
    }

    /// Returns the press timestamp.
    pub const fn press_time(&self) -> u64 {
        self.press_time
    }

    /// Returns the tap key from config.
    pub const fn tap_key(&self) -> KeyCode {
        self.config.tap_key
    }

    /// Returns the hold modifier from config.
    pub const fn hold_modifier(&self) -> u8 {
        self.config.hold_modifier
    }

    /// Returns the threshold in microseconds.
    pub const fn threshold_us(&self) -> u64 {
        self.config.threshold_us
    }

    /// Checks if the threshold has been exceeded at the given time.
    ///
    /// # Arguments
    ///
    /// * `current_time` - Current timestamp in microseconds
    ///
    /// # Returns
    ///
    /// `true` if (current_time - press_time) >= threshold
    pub const fn is_threshold_exceeded(&self, current_time: u64) -> bool {
        current_time.saturating_sub(self.press_time) >= self.config.threshold_us
    }

    /// Calculates elapsed time since press.
    ///
    /// # Arguments
    ///
    /// * `current_time` - Current timestamp in microseconds
    pub const fn elapsed(&self, current_time: u64) -> u64 {
        current_time.saturating_sub(self.press_time)
    }

    // --- State Transitions ---

    /// Transitions from Idle to Pending on key press.
    ///
    /// Records the press timestamp for later threshold checking.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Press event timestamp in microseconds
    ///
    /// # Panics
    ///
    /// Debug asserts that current phase is Idle.
    pub fn transition_to_pending(&mut self, timestamp: u64) {
        debug_assert!(
            self.phase.is_idle(),
            "transition_to_pending called from non-Idle phase: {:?}",
            self.phase
        );
        self.phase = TapHoldPhase::Pending;
        self.press_time = timestamp;
    }

    /// Transitions from Pending to Hold.
    ///
    /// Called when threshold is exceeded or another key interrupts (permissive hold).
    ///
    /// # Panics
    ///
    /// Debug asserts that current phase is Pending.
    pub fn transition_to_hold(&mut self) {
        debug_assert!(
            self.phase.is_pending(),
            "transition_to_hold called from non-Pending phase: {:?}",
            self.phase
        );
        self.phase = TapHoldPhase::Hold;
    }

    /// Transitions back to Idle.
    ///
    /// Called on key release in any active phase.
    pub fn transition_to_idle(&mut self) {
        self.phase = TapHoldPhase::Idle;
        self.press_time = 0;
    }

    /// Resets the state to Idle.
    ///
    /// Same as `transition_to_idle()` but more explicit naming.
    pub fn reset(&mut self) {
        self.transition_to_idle();
    }
}

/// Default maximum concurrent tap-hold keys.
pub const DEFAULT_MAX_PENDING: usize = 32;

/// Result of a timeout check operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeoutResult {
    /// The key that timed out
    pub key: KeyCode,
    /// The hold modifier to activate
    pub hold_modifier: u8,
}

/// Registry for tracking multiple concurrent tap-hold keys.
///
/// Uses a fixed-size `ArrayVec` to avoid heap allocation while supporting
/// multiple simultaneous tap-hold key presses. Keys in `Pending` state are
/// tracked here for timeout checking.
///
/// # Capacity
///
/// The registry has a compile-time maximum capacity specified by the const
/// generic `N`. When full, new entries cannot be added. The default is 32
/// concurrent tap-hold keys, which is more than enough for typical usage.
///
/// # Performance
///
/// - Add: O(1) amortized
/// - Remove: O(n) linear scan
/// - Get/GetMut: O(n) linear scan
/// - Iteration: O(n) cache-friendly
///
/// # Example
///
/// ```rust
/// use keyrx_core::runtime::tap_hold::{PendingKeyRegistry, TapHoldState, TapHoldConfig, DEFAULT_MAX_PENDING};
/// use keyrx_core::config::KeyCode;
///
/// let mut registry: PendingKeyRegistry<DEFAULT_MAX_PENDING> = PendingKeyRegistry::new();
///
/// // Add a pending tap-hold state
/// let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
/// let mut state = TapHoldState::new(KeyCode::CapsLock, config);
/// state.transition_to_pending(1000);
///
/// assert!(registry.add(state));
/// assert_eq!(registry.len(), 1);
///
/// // Look up by key
/// assert!(registry.get(KeyCode::CapsLock).is_some());
///
/// // Remove when done
/// assert!(registry.remove(KeyCode::CapsLock));
/// assert!(registry.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct PendingKeyRegistry<const N: usize = DEFAULT_MAX_PENDING> {
    /// Storage for pending tap-hold states
    entries: ArrayVec<TapHoldState, N>,
}

impl<const N: usize> Default for PendingKeyRegistry<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> PendingKeyRegistry<N> {
    /// Creates a new empty registry.
    pub const fn new() -> Self {
        Self {
            entries: ArrayVec::new_const(),
        }
    }

    /// Returns the number of pending tap-hold keys.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns true if the registry is at capacity.
    pub fn is_full(&self) -> bool {
        self.entries.is_full()
    }

    /// Returns the maximum capacity of the registry.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Adds a tap-hold state to the registry.
    ///
    /// # Returns
    ///
    /// - `true` if the state was added
    /// - `false` if the registry is full or the key already exists
    ///
    /// # Note
    ///
    /// If a state for the same key already exists, the add fails.
    /// Use `remove` first to replace an existing entry.
    pub fn add(&mut self, state: TapHoldState) -> bool {
        // Check if key already exists
        if self.contains(state.key()) {
            return false;
        }

        // Try to add (fails if full)
        self.entries.try_push(state).is_ok()
    }

    /// Removes a tap-hold state by key.
    ///
    /// # Returns
    ///
    /// - `true` if the state was found and removed
    /// - `false` if no state for this key exists
    pub fn remove(&mut self, key: KeyCode) -> bool {
        if let Some(idx) = self.find_index(key) {
            self.entries.swap_remove(idx);
            true
        } else {
            false
        }
    }

    /// Checks if a key is in the registry.
    pub fn contains(&self, key: KeyCode) -> bool {
        self.entries.iter().any(|s| s.key() == key)
    }

    /// Gets an immutable reference to a tap-hold state by key.
    pub fn get(&self, key: KeyCode) -> Option<&TapHoldState> {
        self.entries.iter().find(|s| s.key() == key)
    }

    /// Gets a mutable reference to a tap-hold state by key.
    pub fn get_mut(&mut self, key: KeyCode) -> Option<&mut TapHoldState> {
        self.entries.iter_mut().find(|s| s.key() == key)
    }

    /// Returns an iterator over all pending states.
    pub fn iter(&self) -> impl Iterator<Item = &TapHoldState> {
        self.entries.iter()
    }

    /// Returns a mutable iterator over all pending states.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TapHoldState> {
        self.entries.iter_mut()
    }

    /// Clears all entries from the registry.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Checks for timeouts and returns keys that exceeded their thresholds.
    ///
    /// This method checks all pending keys against the current time and:
    /// 1. Returns information about keys that exceeded their threshold
    /// 2. Transitions those keys to Hold state
    ///
    /// # Arguments
    ///
    /// * `current_time` - Current timestamp in microseconds
    ///
    /// # Returns
    ///
    /// An `ArrayVec` of `TimeoutResult` for keys that timed out.
    /// Maximum size matches the registry capacity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use keyrx_core::runtime::tap_hold::{PendingKeyRegistry, TapHoldState, TapHoldConfig, DEFAULT_MAX_PENDING};
    /// use keyrx_core::config::KeyCode;
    ///
    /// let mut registry: PendingKeyRegistry<DEFAULT_MAX_PENDING> = PendingKeyRegistry::new();
    ///
    /// // Add a pending state
    /// let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000); // 200ms threshold
    /// let mut state = TapHoldState::new(KeyCode::CapsLock, config);
    /// state.transition_to_pending(0); // pressed at time 0
    /// registry.add(state);
    ///
    /// // Before threshold - no timeouts
    /// let timeouts = registry.check_timeouts(100_000); // 100ms
    /// assert!(timeouts.is_empty());
    ///
    /// // After threshold - CapsLock times out
    /// let timeouts = registry.check_timeouts(300_000); // 300ms
    /// assert_eq!(timeouts.len(), 1);
    /// assert_eq!(timeouts[0].key, KeyCode::CapsLock);
    /// assert_eq!(timeouts[0].hold_modifier, 0);
    /// ```
    pub fn check_timeouts(&mut self, current_time: u64) -> ArrayVec<TimeoutResult, N> {
        let mut results = ArrayVec::new();

        for state in self.entries.iter_mut() {
            // Only check Pending states
            if !state.phase().is_pending() {
                continue;
            }

            // Check if threshold exceeded
            if state.is_threshold_exceeded(current_time) {
                // Transition to Hold
                state.transition_to_hold();

                // Record the timeout
                // This won't fail since results capacity matches entries capacity
                let _ = results.try_push(TimeoutResult {
                    key: state.key(),
                    hold_modifier: state.hold_modifier(),
                });
            }
        }

        results
    }

    /// Finds all keys in Pending state.
    ///
    /// Useful for checking if any keys might need permissive hold activation.
    pub fn pending_keys(&self) -> impl Iterator<Item = &TapHoldState> {
        self.entries.iter().filter(|s| s.phase().is_pending())
    }

    /// Triggers permissive hold for all pending keys.
    ///
    /// When another key is pressed while tap-hold keys are pending,
    /// this method transitions all pending keys to Hold state immediately.
    ///
    /// # Returns
    ///
    /// Information about all keys that were transitioned to Hold.
    pub fn trigger_permissive_hold(&mut self) -> ArrayVec<TimeoutResult, N> {
        let mut results = ArrayVec::new();

        for state in self.entries.iter_mut() {
            if state.phase().is_pending() {
                state.transition_to_hold();

                let _ = results.try_push(TimeoutResult {
                    key: state.key(),
                    hold_modifier: state.hold_modifier(),
                });
            }
        }

        results
    }

    // --- Private helpers ---

    /// Finds the index of a state by key.
    fn find_index(&self, key: KeyCode) -> Option<usize> {
        self.entries.iter().position(|s| s.key() == key)
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::vec::Vec;

    use super::*;

    // --- TapHoldPhase Tests ---

    #[test]
    fn test_phase_default_is_idle() {
        let phase = TapHoldPhase::default();
        assert_eq!(phase, TapHoldPhase::Idle);
    }

    #[test]
    fn test_phase_is_idle() {
        assert!(TapHoldPhase::Idle.is_idle());
        assert!(!TapHoldPhase::Pending.is_idle());
        assert!(!TapHoldPhase::Hold.is_idle());
    }

    #[test]
    fn test_phase_is_pending() {
        assert!(!TapHoldPhase::Idle.is_pending());
        assert!(TapHoldPhase::Pending.is_pending());
        assert!(!TapHoldPhase::Hold.is_pending());
    }

    #[test]
    fn test_phase_is_hold() {
        assert!(!TapHoldPhase::Idle.is_hold());
        assert!(!TapHoldPhase::Pending.is_hold());
        assert!(TapHoldPhase::Hold.is_hold());
    }

    // --- TapHoldConfig Tests ---

    #[test]
    fn test_config_new() {
        let config = TapHoldConfig::new(KeyCode::Escape, 5, 200_000);

        assert_eq!(config.tap_key(), KeyCode::Escape);
        assert_eq!(config.hold_modifier(), 5);
        assert_eq!(config.threshold_us(), 200_000);
    }

    #[test]
    fn test_config_from_ms() {
        let config = TapHoldConfig::from_ms(KeyCode::Tab, 0, 150);

        assert_eq!(config.tap_key(), KeyCode::Tab);
        assert_eq!(config.hold_modifier(), 0);
        assert_eq!(config.threshold_us(), 150_000);
    }

    #[test]
    fn test_config_from_ms_max_value() {
        // u16::MAX = 65535ms = 65,535,000μs
        let config = TapHoldConfig::from_ms(KeyCode::A, 254, u16::MAX);
        assert_eq!(config.threshold_us(), 65_535_000);
    }

    // --- TapHoldState Tests ---

    #[test]
    fn test_state_new() {
        let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
        let state = TapHoldState::new(KeyCode::CapsLock, config);

        assert_eq!(state.key(), KeyCode::CapsLock);
        assert_eq!(state.phase(), TapHoldPhase::Idle);
        assert_eq!(state.press_time(), 0);
        assert_eq!(state.tap_key(), KeyCode::Escape);
        assert_eq!(state.hold_modifier(), 0);
        assert_eq!(state.threshold_us(), 200_000);
    }

    #[test]
    fn test_state_transition_idle_to_pending() {
        let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        state.transition_to_pending(1000);

        assert_eq!(state.phase(), TapHoldPhase::Pending);
        assert_eq!(state.press_time(), 1000);
    }

    #[test]
    fn test_state_transition_pending_to_hold() {
        let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        state.transition_to_pending(1000);
        state.transition_to_hold();

        assert_eq!(state.phase(), TapHoldPhase::Hold);
        assert_eq!(state.press_time(), 1000); // press_time preserved
    }

    #[test]
    fn test_state_transition_to_idle() {
        let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        // From Pending
        state.transition_to_pending(1000);
        state.transition_to_idle();
        assert_eq!(state.phase(), TapHoldPhase::Idle);
        assert_eq!(state.press_time(), 0);

        // From Hold
        state.transition_to_pending(2000);
        state.transition_to_hold();
        state.transition_to_idle();
        assert_eq!(state.phase(), TapHoldPhase::Idle);
        assert_eq!(state.press_time(), 0);
    }

    #[test]
    fn test_state_reset() {
        let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        state.transition_to_pending(5000);
        state.reset();

        assert_eq!(state.phase(), TapHoldPhase::Idle);
        assert_eq!(state.press_time(), 0);
    }

    #[test]
    fn test_is_threshold_exceeded() {
        let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000); // 200ms
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        state.transition_to_pending(1_000_000); // pressed at 1s

        // Before threshold
        assert!(!state.is_threshold_exceeded(1_100_000)); // 100ms elapsed
        assert!(!state.is_threshold_exceeded(1_199_999)); // just under

        // At threshold
        assert!(state.is_threshold_exceeded(1_200_000)); // exactly 200ms

        // After threshold
        assert!(state.is_threshold_exceeded(1_300_000)); // 300ms elapsed
    }

    #[test]
    fn test_elapsed() {
        let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        state.transition_to_pending(1_000_000);

        assert_eq!(state.elapsed(1_000_000), 0);
        assert_eq!(state.elapsed(1_100_000), 100_000);
        assert_eq!(state.elapsed(1_500_000), 500_000);
    }

    #[test]
    fn test_elapsed_saturates_on_underflow() {
        let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        state.transition_to_pending(1_000_000);

        // Current time before press time (shouldn't happen, but handle gracefully)
        assert_eq!(state.elapsed(500_000), 0);
    }

    #[test]
    fn test_tap_scenario() {
        let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        // Key pressed
        state.transition_to_pending(0);
        assert!(state.phase().is_pending());

        // Quick release (100ms < 200ms threshold)
        let release_time = 100_000; // 100ms
        assert!(!state.is_threshold_exceeded(release_time));

        // Would emit tap key (Escape) - transition back to idle
        state.transition_to_idle();
        assert!(state.phase().is_idle());
    }

    #[test]
    fn test_hold_scenario() {
        let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        // Key pressed
        state.transition_to_pending(0);
        assert!(state.phase().is_pending());

        // Threshold exceeded (300ms > 200ms)
        let check_time = 300_000;
        assert!(state.is_threshold_exceeded(check_time));

        // Transition to hold
        state.transition_to_hold();
        assert!(state.phase().is_hold());

        // Key released
        state.transition_to_idle();
        assert!(state.phase().is_idle());
    }

    #[test]
    fn test_permissive_hold_scenario() {
        let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        // Key pressed
        state.transition_to_pending(0);

        // Another key pressed before threshold (50ms < 200ms)
        // This triggers permissive hold
        let interrupt_time = 50_000;
        assert!(!state.is_threshold_exceeded(interrupt_time));

        // Immediately transition to hold (permissive hold behavior)
        state.transition_to_hold();
        assert!(state.phase().is_hold());
    }

    #[test]
    fn test_config_accessors() {
        let config = TapHoldConfig::new(KeyCode::Tab, 3, 150_000);
        let state = TapHoldState::new(KeyCode::Space, config);

        assert_eq!(state.config().tap_key(), KeyCode::Tab);
        assert_eq!(state.config().hold_modifier(), 3);
        assert_eq!(state.config().threshold_us(), 150_000);
    }

    // --- PendingKeyRegistry Tests ---

    /// Helper to create a pending state for testing
    fn make_pending_state(
        key: KeyCode,
        tap: KeyCode,
        modifier: u8,
        press_time: u64,
    ) -> TapHoldState {
        let config = TapHoldConfig::from_ms(tap, modifier, 200);
        let mut state = TapHoldState::new(key, config);
        state.transition_to_pending(press_time);
        state
    }

    #[test]
    fn test_registry_new() {
        let registry: PendingKeyRegistry<32> = PendingKeyRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(!registry.is_full());
        assert_eq!(registry.capacity(), 32);
    }

    #[test]
    fn test_registry_default() {
        let registry: PendingKeyRegistry<8> = PendingKeyRegistry::default();
        assert!(registry.is_empty());
        assert_eq!(registry.capacity(), 8);
    }

    #[test]
    fn test_registry_add_single() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();
        let state = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);

        assert!(registry.add(state));
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
        assert!(registry.contains(KeyCode::CapsLock));
    }

    #[test]
    fn test_registry_add_multiple() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

        let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
        let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 2000);
        let state3 = make_pending_state(KeyCode::Space, KeyCode::Space, 2, 3000);

        assert!(registry.add(state1));
        assert!(registry.add(state2));
        assert!(registry.add(state3));

        assert_eq!(registry.len(), 3);
        assert!(registry.contains(KeyCode::CapsLock));
        assert!(registry.contains(KeyCode::Tab));
        assert!(registry.contains(KeyCode::Space));
    }

    #[test]
    fn test_registry_add_duplicate_fails() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

        let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
        let state2 = make_pending_state(KeyCode::CapsLock, KeyCode::Tab, 1, 2000);

        assert!(registry.add(state1));
        assert!(!registry.add(state2)); // Same key, should fail
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_registry_add_when_full() {
        let mut registry: PendingKeyRegistry<2> = PendingKeyRegistry::new();

        let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
        let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 2000);
        let state3 = make_pending_state(KeyCode::Space, KeyCode::Space, 2, 3000);

        assert!(registry.add(state1));
        assert!(registry.add(state2));
        assert!(registry.is_full());
        assert!(!registry.add(state3)); // Should fail, registry full
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_registry_remove() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

        let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
        let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 2000);

        registry.add(state1);
        registry.add(state2);

        assert!(registry.remove(KeyCode::CapsLock));
        assert_eq!(registry.len(), 1);
        assert!(!registry.contains(KeyCode::CapsLock));
        assert!(registry.contains(KeyCode::Tab));
    }

    #[test]
    fn test_registry_remove_nonexistent() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();
        let state = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);

        registry.add(state);
        assert!(!registry.remove(KeyCode::Tab)); // Not in registry
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_registry_get() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();
        let state = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
        registry.add(state);

        let retrieved = registry.get(KeyCode::CapsLock);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().key(), KeyCode::CapsLock);
        assert_eq!(retrieved.unwrap().tap_key(), KeyCode::Escape);

        assert!(registry.get(KeyCode::Tab).is_none());
    }

    #[test]
    fn test_registry_get_mut() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();
        let state = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
        registry.add(state);

        // Modify the state
        if let Some(s) = registry.get_mut(KeyCode::CapsLock) {
            s.transition_to_hold();
        }

        let retrieved = registry.get(KeyCode::CapsLock);
        assert!(retrieved.unwrap().phase().is_hold());
    }

    #[test]
    fn test_registry_iter() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

        let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
        let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 2000);

        registry.add(state1);
        registry.add(state2);

        let keys: Vec<_> = registry.iter().map(|s| s.key()).collect();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&KeyCode::CapsLock));
        assert!(keys.contains(&KeyCode::Tab));
    }

    #[test]
    fn test_registry_iter_mut() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

        let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
        let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 2000);

        registry.add(state1);
        registry.add(state2);

        // Transition all to hold
        for s in registry.iter_mut() {
            s.transition_to_hold();
        }

        // Verify all are in Hold state
        for s in registry.iter() {
            assert!(s.phase().is_hold());
        }
    }

    #[test]
    fn test_registry_clear() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

        let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 1000);
        let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 2000);

        registry.add(state1);
        registry.add(state2);
        assert_eq!(registry.len(), 2);

        registry.clear();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_check_timeouts_no_timeouts() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

        // 200ms threshold, pressed at time 0
        let state = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
        registry.add(state);

        // Check at 100ms - no timeout
        let results = registry.check_timeouts(100_000);
        assert!(results.is_empty());

        // State should still be Pending
        assert!(registry
            .get(KeyCode::CapsLock)
            .unwrap()
            .phase()
            .is_pending());
    }

    #[test]
    fn test_registry_check_timeouts_single_timeout() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

        // 200ms threshold, pressed at time 0
        let state = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
        registry.add(state);

        // Check at 300ms - timeout!
        let results = registry.check_timeouts(300_000);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, KeyCode::CapsLock);
        assert_eq!(results[0].hold_modifier, 0);

        // State should now be Hold
        assert!(registry.get(KeyCode::CapsLock).unwrap().phase().is_hold());
    }

    #[test]
    fn test_registry_check_timeouts_at_exact_threshold() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

        // 200ms threshold, pressed at time 0
        let state = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
        registry.add(state);

        // Check at exactly 200ms - should timeout
        let results = registry.check_timeouts(200_000);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_registry_check_timeouts_multiple() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

        // CapsLock: 200ms threshold, pressed at 0
        let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
        // Tab: 200ms threshold, pressed at 100ms
        let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 100_000);

        registry.add(state1);
        registry.add(state2);

        // At 250ms: CapsLock times out (250ms > 200ms), Tab doesn't (150ms < 200ms)
        let results = registry.check_timeouts(250_000);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, KeyCode::CapsLock);

        // At 350ms: Tab also times out (250ms > 200ms)
        let results = registry.check_timeouts(350_000);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, KeyCode::Tab);
    }

    #[test]
    fn test_registry_check_timeouts_ignores_hold_state() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

        let state = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
        registry.add(state);

        // First timeout check
        let results = registry.check_timeouts(300_000);
        assert_eq!(results.len(), 1);

        // Second timeout check - should not report again since now in Hold state
        let results = registry.check_timeouts(400_000);
        assert!(results.is_empty());
    }

    #[test]
    fn test_registry_pending_keys() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

        let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
        let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 0);

        registry.add(state1);
        registry.add(state2);

        // Both pending initially
        assert_eq!(registry.pending_keys().count(), 2);

        // Transition one to Hold
        if let Some(s) = registry.get_mut(KeyCode::CapsLock) {
            s.transition_to_hold();
        }

        // Only one pending now
        assert_eq!(registry.pending_keys().count(), 1);
        let pending: Vec<_> = registry.pending_keys().collect();
        assert_eq!(pending[0].key(), KeyCode::Tab);
    }

    #[test]
    fn test_registry_trigger_permissive_hold() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

        let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
        let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 0);

        registry.add(state1);
        registry.add(state2);

        // Trigger permissive hold for all
        let results = registry.trigger_permissive_hold();

        // Both should be reported
        assert_eq!(results.len(), 2);

        // Both should now be in Hold state
        assert!(registry.get(KeyCode::CapsLock).unwrap().phase().is_hold());
        assert!(registry.get(KeyCode::Tab).unwrap().phase().is_hold());
    }

    #[test]
    fn test_registry_trigger_permissive_hold_ignores_hold() {
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

        let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
        let state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 0);

        registry.add(state1);
        registry.add(state2);

        // Manually transition one to Hold
        registry
            .get_mut(KeyCode::CapsLock)
            .unwrap()
            .transition_to_hold();

        // Trigger permissive hold
        let results = registry.trigger_permissive_hold();

        // Only Tab should be reported (CapsLock was already Hold)
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, KeyCode::Tab);
    }

    #[test]
    fn test_registry_concurrent_keys_scenario() {
        // Simulate realistic concurrent tap-hold usage:
        // User presses CapsLock(hold=Ctrl), then Tab(hold=Alt), then releases both
        let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

        // CapsLock pressed at time 0
        let caps = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
        registry.add(caps);
        assert_eq!(registry.len(), 1);

        // Tab pressed at time 100ms
        let tab = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 100_000);
        registry.add(tab);
        assert_eq!(registry.len(), 2);

        // Check timeouts at 150ms - neither has timed out yet (200ms threshold)
        let results = registry.check_timeouts(150_000);
        assert!(results.is_empty());

        // Check timeouts at 250ms - CapsLock times out (250ms > 200ms)
        // Tab has only 150ms elapsed (250ms - 100ms), still pending
        let results = registry.check_timeouts(250_000);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, KeyCode::CapsLock);

        // Check timeouts at 350ms - Tab times out too (250ms elapsed > 200ms)
        let results = registry.check_timeouts(350_000);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, KeyCode::Tab);

        // Both in Hold state, user types 'A' - gets Ctrl+Alt+A
        // Then releases both
        registry.remove(KeyCode::CapsLock);
        registry.remove(KeyCode::Tab);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_timeout_result_fields() {
        let result = TimeoutResult {
            key: KeyCode::CapsLock,
            hold_modifier: 5,
        };

        assert_eq!(result.key, KeyCode::CapsLock);
        assert_eq!(result.hold_modifier, 5);

        // Test Clone and Copy
        let result2 = result;
        assert_eq!(result, result2);
    }

    #[test]
    fn test_registry_with_different_capacities() {
        // Small capacity
        let small: PendingKeyRegistry<2> = PendingKeyRegistry::new();
        assert_eq!(small.capacity(), 2);

        // Default capacity
        let default: PendingKeyRegistry<DEFAULT_MAX_PENDING> = PendingKeyRegistry::new();
        assert_eq!(default.capacity(), 32);

        // Large capacity
        let large: PendingKeyRegistry<64> = PendingKeyRegistry::new();
        assert_eq!(large.capacity(), 64);
    }
}
