//! Device state management with bit vectors
//!
//! This module provides `DeviceState` for tracking modifier and lock state
//! using efficient 255-bit vectors.

use bitvec::prelude::*;

/// Maximum valid modifier/lock ID (0-254, ID 255 is reserved)
const MAX_VALID_ID: u8 = 254;

/// Device state tracking modifier and lock state
///
/// Uses 255-bit vectors for efficient state management:
/// - Modifiers: Temporary state (set on press, clear on release)
/// - Locks: Toggle state (toggle on press, ignore release)
///
/// Bit layout: IDs 0-254 are valid, ID 255 is reserved and will be rejected.
///
/// # Example
///
/// ```rust,ignore
/// use keyrx_core::runtime::DeviceState;
///
/// let mut state = DeviceState::new();
/// state.set_modifier(0);
/// assert!(state.is_modifier_active(0));
/// ```
pub struct DeviceState {
    /// Modifier state (255 bits, IDs 0-254)
    modifiers: BitVec<u8, Lsb0>,
    /// Lock state (255 bits, IDs 0-254)
    locks: BitVec<u8, Lsb0>,
}

impl DeviceState {
    /// Creates a new device state with all bits cleared
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let state = DeviceState::new();
    /// assert!(!state.is_modifier_active(0));
    /// assert!(!state.is_lock_active(0));
    /// ```
    pub fn new() -> Self {
        Self {
            modifiers: bitvec![u8, Lsb0; 0; 255],
            locks: bitvec![u8, Lsb0; 0; 255],
        }
    }

    /// Validates that a modifier/lock ID is in valid range (0-254)
    ///
    /// Returns true if valid, logs error and returns false if invalid (>254).
    #[inline]
    fn validate_id(id: u8) -> bool {
        if id > MAX_VALID_ID {
            // In production, this would use proper logging
            // For now, we just return false
            false
        } else {
            true
        }
    }

    /// Sets a modifier bit to active
    ///
    /// # Arguments
    ///
    /// * `id` - Modifier ID (0-254)
    ///
    /// # Returns
    ///
    /// Returns `true` if successful, `false` if ID is invalid (>254)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut state = DeviceState::new();
    /// assert!(state.set_modifier(0));
    /// assert!(state.is_modifier_active(0));
    /// assert!(!state.set_modifier(255)); // Invalid ID
    /// ```
    pub fn set_modifier(&mut self, id: u8) -> bool {
        if !Self::validate_id(id) {
            return false;
        }
        self.modifiers.set(id as usize, true);
        true
    }

    /// Clears a modifier bit to inactive
    ///
    /// # Arguments
    ///
    /// * `id` - Modifier ID (0-254)
    ///
    /// # Returns
    ///
    /// Returns `true` if successful, `false` if ID is invalid (>254)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut state = DeviceState::new();
    /// state.set_modifier(0);
    /// assert!(state.clear_modifier(0));
    /// assert!(!state.is_modifier_active(0));
    /// ```
    pub fn clear_modifier(&mut self, id: u8) -> bool {
        if !Self::validate_id(id) {
            return false;
        }
        self.modifiers.set(id as usize, false);
        true
    }

    /// Toggles a lock bit (OFF→ON or ON→OFF)
    ///
    /// # Arguments
    ///
    /// * `id` - Lock ID (0-254)
    ///
    /// # Returns
    ///
    /// Returns `true` if successful, `false` if ID is invalid (>254)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut state = DeviceState::new();
    /// assert!(state.toggle_lock(0)); // OFF → ON
    /// assert!(state.is_lock_active(0));
    /// assert!(state.toggle_lock(0)); // ON → OFF
    /// assert!(!state.is_lock_active(0));
    /// ```
    pub fn toggle_lock(&mut self, id: u8) -> bool {
        if !Self::validate_id(id) {
            return false;
        }
        let current = self.locks[id as usize];
        self.locks.set(id as usize, !current);
        true
    }

    /// Checks if a modifier is active
    ///
    /// # Arguments
    ///
    /// * `id` - Modifier ID (0-254)
    ///
    /// # Returns
    ///
    /// Returns `true` if modifier is active, `false` if inactive or ID is invalid
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut state = DeviceState::new();
    /// assert!(!state.is_modifier_active(0));
    /// state.set_modifier(0);
    /// assert!(state.is_modifier_active(0));
    /// ```
    pub fn is_modifier_active(&self, id: u8) -> bool {
        if !Self::validate_id(id) {
            return false;
        }
        self.modifiers[id as usize]
    }

    /// Checks if a lock is active
    ///
    /// # Arguments
    ///
    /// * `id` - Lock ID (0-254)
    ///
    /// # Returns
    ///
    /// Returns `true` if lock is active, `false` if inactive or ID is invalid
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut state = DeviceState::new();
    /// assert!(!state.is_lock_active(0));
    /// state.toggle_lock(0);
    /// assert!(state.is_lock_active(0));
    /// ```
    pub fn is_lock_active(&self, id: u8) -> bool {
        if !Self::validate_id(id) {
            return false;
        }
        self.locks[id as usize]
    }
}

impl Default for DeviceState {
    fn default() -> Self {
        Self::new()
    }
}
