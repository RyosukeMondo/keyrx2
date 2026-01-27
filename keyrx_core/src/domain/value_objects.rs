//! Domain value objects for Core domain
//!
//! Value objects are immutable and defined by their attributes, not identity.

use crate::config::{Condition, KeyCode};
use crate::runtime::DeviceState;

/// KeyCode value object
///
/// Wraps the enum KeyCode as a value object with additional validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyCodeVO(KeyCode);

impl KeyCodeVO {
    /// Creates a new KeyCode value object
    pub fn new(code: KeyCode) -> Self {
        Self(code)
    }

    /// Gets the inner KeyCode
    pub fn inner(&self) -> KeyCode {
        self.0
    }

    /// Checks if this is a modifier key (Shift, Ctrl, Alt, Win)
    pub fn is_modifier(&self) -> bool {
        matches!(
            self.0,
            KeyCode::LShift
                | KeyCode::RShift
                | KeyCode::LCtrl
                | KeyCode::RCtrl
                | KeyCode::LAlt
                | KeyCode::RAlt
                | KeyCode::LMeta
                | KeyCode::RMeta
        )
    }

    /// Checks if this is a lock key (CapsLock, NumLock, ScrollLock)
    pub fn is_lock(&self) -> bool {
        matches!(
            self.0,
            KeyCode::CapsLock | KeyCode::NumLock | KeyCode::ScrollLock
        )
    }
}

impl From<KeyCode> for KeyCodeVO {
    fn from(code: KeyCode) -> Self {
        Self(code)
    }
}

/// Condition value object
///
/// Wraps Condition with domain-specific behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionVO {
    inner: Condition,
}

impl ConditionVO {
    /// Creates a new Condition value object
    pub fn new(condition: Condition) -> Self {
        Self { inner: condition }
    }

    /// Evaluates this condition against a state
    pub fn evaluate(&self, state: &DeviceState) -> bool {
        state.evaluate_condition(&self.inner)
    }

    /// Checks if this is a simple modifier condition
    pub fn is_modifier_active(&self) -> bool {
        matches!(self.inner, Condition::ModifierActive(_))
    }

    /// Checks if this is a simple lock condition
    pub fn is_lock_active(&self) -> bool {
        matches!(self.inner, Condition::LockActive(_))
    }

    /// Checks if this is an AllActive (AND) condition
    pub fn is_all_active(&self) -> bool {
        matches!(self.inner, Condition::AllActive(_))
    }

    /// Checks if this is a NotActive (negated) condition
    pub fn is_not_active(&self) -> bool {
        matches!(self.inner, Condition::NotActive(_))
    }

    /// Checks if this is a device matching condition
    pub fn is_device_matches(&self) -> bool {
        matches!(self.inner, Condition::DeviceMatches(_))
    }

    /// Gets the inner condition
    pub fn inner(&self) -> &Condition {
        &self.inner
    }
}

impl From<Condition> for ConditionVO {
    fn from(condition: Condition) -> Self {
        Self::new(condition)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_key_code_vo_modifiers() {
        let shift = KeyCodeVO::new(KeyCode::LShift);
        assert!(shift.is_modifier());
        assert!(!shift.is_lock());

        let caps = KeyCodeVO::new(KeyCode::CapsLock);
        assert!(!caps.is_modifier());
        assert!(caps.is_lock());

        let a = KeyCodeVO::new(KeyCode::A);
        assert!(!a.is_modifier());
        assert!(!a.is_lock());
    }

    #[test]
    fn test_key_code_vo_equality() {
        let a1 = KeyCodeVO::new(KeyCode::A);
        let a2 = KeyCodeVO::new(KeyCode::A);
        let b = KeyCodeVO::new(KeyCode::B);

        assert_eq!(a1, a2);
        assert_ne!(a1, b);
    }

    #[test]
    fn test_condition_vo_types() {
        let modifier = ConditionVO::new(Condition::ModifierActive(0));
        assert!(modifier.is_modifier_active());
        assert!(!modifier.is_lock_active());

        let lock = ConditionVO::new(Condition::LockActive(1));
        assert!(!lock.is_modifier_active());
        assert!(lock.is_lock_active());

        let all_active = ConditionVO::new(Condition::AllActive(vec![]));
        assert!(all_active.is_all_active());
        assert!(!all_active.is_not_active());

        let not_active = ConditionVO::new(Condition::NotActive(vec![]));
        assert!(!not_active.is_all_active());
        assert!(not_active.is_not_active());
    }
}
