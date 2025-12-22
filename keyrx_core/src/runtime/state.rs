//! Device state management with bit vectors
//!
//! This module provides `DeviceState` for tracking modifier and lock state
//! using efficient 255-bit vectors.

use bitvec::prelude::*;

use crate::config::{Condition, ConditionItem};

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

    /// Evaluates a condition against the current device state
    ///
    /// # Arguments
    ///
    /// * `condition` - The condition to evaluate
    ///
    /// # Returns
    ///
    /// Returns `true` if the condition is satisfied, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use keyrx_core::runtime::DeviceState;
    /// use keyrx_core::config::{Condition, ConditionItem};
    ///
    /// let mut state = DeviceState::new();
    /// state.set_modifier(0);
    ///
    /// // Single modifier active
    /// assert!(state.evaluate_condition(&Condition::ModifierActive(0)));
    ///
    /// // All conditions must be true
    /// state.toggle_lock(1);
    /// let all_cond = Condition::AllActive(vec![
    ///     ConditionItem::ModifierActive(0),
    ///     ConditionItem::LockActive(1),
    /// ]);
    /// assert!(state.evaluate_condition(&all_cond));
    ///
    /// // Not active
    /// let not_cond = Condition::NotActive(vec![ConditionItem::ModifierActive(2)]);
    /// assert!(state.evaluate_condition(&not_cond)); // MD_02 is not active
    /// ```
    pub fn evaluate_condition(&self, condition: &Condition) -> bool {
        match condition {
            // Single modifier active
            Condition::ModifierActive(id) => self.is_modifier_active(*id),

            // Single lock active
            Condition::LockActive(id) => self.is_lock_active(*id),

            // All conditions must be true (AND logic)
            Condition::AllActive(items) => {
                items.iter().all(|item| self.evaluate_condition_item(item))
            }

            // All conditions must be false (NOT logic)
            Condition::NotActive(items) => {
                items.iter().all(|item| !self.evaluate_condition_item(item))
            }
        }
    }

    /// Evaluates a single condition item
    ///
    /// Helper method for evaluating ConditionItem in composite conditions.
    fn evaluate_condition_item(&self, item: &ConditionItem) -> bool {
        match item {
            ConditionItem::ModifierActive(id) => self.is_modifier_active(*id),
            ConditionItem::LockActive(id) => self.is_lock_active(*id),
        }
    }
}

impl Default for DeviceState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::vec;

    #[test]
    fn test_new_creates_zeroed_state() {
        let state = DeviceState::new();
        assert!(!state.is_modifier_active(0));
        assert!(!state.is_modifier_active(127));
        assert!(!state.is_modifier_active(254));
        assert!(!state.is_lock_active(0));
        assert!(!state.is_lock_active(127));
        assert!(!state.is_lock_active(254));
    }

    #[test]
    fn test_set_modifier_valid_ids() {
        let mut state = DeviceState::new();

        // Test ID 0
        assert!(state.set_modifier(0));
        assert!(state.is_modifier_active(0));

        // Test ID 127 (middle)
        assert!(state.set_modifier(127));
        assert!(state.is_modifier_active(127));

        // Test ID 254 (max valid)
        assert!(state.set_modifier(254));
        assert!(state.is_modifier_active(254));
    }

    #[test]
    fn test_set_modifier_invalid_id() {
        let mut state = DeviceState::new();

        // ID 255 should be rejected
        assert!(!state.set_modifier(255));
        // Modifier should not be set
        assert!(!state.is_modifier_active(255));
    }

    #[test]
    fn test_clear_modifier() {
        let mut state = DeviceState::new();

        // Set then clear
        state.set_modifier(0);
        assert!(state.is_modifier_active(0));
        assert!(state.clear_modifier(0));
        assert!(!state.is_modifier_active(0));

        // Clear invalid ID
        assert!(!state.clear_modifier(255));
    }

    #[test]
    fn test_toggle_lock_cycles() {
        let mut state = DeviceState::new();

        // OFF → ON
        assert!(state.toggle_lock(0));
        assert!(state.is_lock_active(0));

        // ON → OFF
        assert!(state.toggle_lock(0));
        assert!(!state.is_lock_active(0));

        // OFF → ON again
        assert!(state.toggle_lock(0));
        assert!(state.is_lock_active(0));
    }

    #[test]
    fn test_toggle_lock_invalid_id() {
        let mut state = DeviceState::new();

        // ID 255 should be rejected
        assert!(!state.toggle_lock(255));
        assert!(!state.is_lock_active(255));
    }

    #[test]
    fn test_evaluate_condition_modifier_active() {
        let mut state = DeviceState::new();
        state.set_modifier(0);

        let cond = Condition::ModifierActive(0);
        assert!(state.evaluate_condition(&cond));

        let cond_inactive = Condition::ModifierActive(1);
        assert!(!state.evaluate_condition(&cond_inactive));
    }

    #[test]
    fn test_evaluate_condition_lock_active() {
        let mut state = DeviceState::new();
        state.toggle_lock(1);

        let cond = Condition::LockActive(1);
        assert!(state.evaluate_condition(&cond));

        let cond_inactive = Condition::LockActive(2);
        assert!(!state.evaluate_condition(&cond_inactive));
    }

    #[test]
    fn test_evaluate_condition_all_active() {
        let mut state = DeviceState::new();
        state.set_modifier(0);
        state.toggle_lock(1);

        // Both conditions true
        let cond = Condition::AllActive(vec![
            ConditionItem::ModifierActive(0),
            ConditionItem::LockActive(1),
        ]);
        assert!(state.evaluate_condition(&cond));

        // One condition false
        let cond_partial = Condition::AllActive(vec![
            ConditionItem::ModifierActive(0),
            ConditionItem::LockActive(2), // Not active
        ]);
        assert!(!state.evaluate_condition(&cond_partial));

        // All conditions false
        let cond_none = Condition::AllActive(vec![
            ConditionItem::ModifierActive(10),
            ConditionItem::LockActive(11),
        ]);
        assert!(!state.evaluate_condition(&cond_none));
    }

    #[test]
    fn test_evaluate_condition_not_active() {
        let mut state = DeviceState::new();
        state.set_modifier(0);

        // NOT(inactive) = true
        let cond_true = Condition::NotActive(vec![ConditionItem::ModifierActive(1)]);
        assert!(state.evaluate_condition(&cond_true));

        // NOT(active) = false
        let cond_false = Condition::NotActive(vec![ConditionItem::ModifierActive(0)]);
        assert!(!state.evaluate_condition(&cond_false));

        // NOT(MD_00 AND LK_01) with MD_00 active, LK_01 inactive = false (not all inactive)
        let cond_mixed = Condition::NotActive(vec![
            ConditionItem::ModifierActive(0), // Active
            ConditionItem::LockActive(1),     // Inactive
        ]);
        assert!(!state.evaluate_condition(&cond_mixed));

        // NOT(MD_10 AND LK_11) with both inactive = true
        let cond_both_inactive = Condition::NotActive(vec![
            ConditionItem::ModifierActive(10),
            ConditionItem::LockActive(11),
        ]);
        assert!(state.evaluate_condition(&cond_both_inactive));
    }

    #[test]
    fn test_multiple_modifiers_independent() {
        let mut state = DeviceState::new();

        state.set_modifier(0);
        state.set_modifier(1);
        state.set_modifier(254);

        assert!(state.is_modifier_active(0));
        assert!(state.is_modifier_active(1));
        assert!(state.is_modifier_active(254));

        state.clear_modifier(1);
        assert!(state.is_modifier_active(0));
        assert!(!state.is_modifier_active(1));
        assert!(state.is_modifier_active(254));
    }

    #[test]
    fn test_multiple_locks_independent() {
        let mut state = DeviceState::new();

        state.toggle_lock(0); // ON
        state.toggle_lock(1); // ON
        state.toggle_lock(2); // ON

        assert!(state.is_lock_active(0));
        assert!(state.is_lock_active(1));
        assert!(state.is_lock_active(2));

        state.toggle_lock(1); // OFF
        assert!(state.is_lock_active(0));
        assert!(!state.is_lock_active(1));
        assert!(state.is_lock_active(2));
    }
}
