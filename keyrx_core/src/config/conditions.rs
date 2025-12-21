use alloc::vec::Vec;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

/// Basic condition check for a single modifier or lock
///
/// Used in composite conditions.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
pub enum ConditionItem {
    /// Custom modifier is active (MD_XX)
    ModifierActive(u8),
    /// Custom lock is active (LK_XX)
    LockActive(u8),
}

/// Conditional mapping support for when/when_not blocks
///
/// Supports single conditions, AND combinations, and negation.
/// To avoid recursive Box issues with rkyv, NotActive contains a Vec
/// of conditions which must ALL be false (implemented as NOT(AND(...))).
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
pub enum Condition {
    /// Single custom modifier active (MD_XX)
    ModifierActive(u8),
    /// Single custom lock active (LK_XX)
    LockActive(u8),
    /// All conditions must be true (AND logic) - for when() with multiple conditions
    AllActive(Vec<ConditionItem>),
    /// All conditions must be false (when_not with AND logic) - negated AllActive
    /// For single condition negation, use vec with one item
    NotActive(Vec<ConditionItem>),
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;

    #[test]
    fn test_condition_variants() {
        // Test ModifierActive variant
        let cond1 = Condition::ModifierActive(0x01);
        assert_eq!(cond1, Condition::ModifierActive(0x01));

        // Test LockActive variant
        let cond2 = Condition::LockActive(0x02);
        assert_eq!(cond2, Condition::LockActive(0x02));

        // Test AllActive variant with multiple conditions
        let cond3 = Condition::AllActive(alloc::vec![
            ConditionItem::ModifierActive(0x01),
            ConditionItem::LockActive(0x02),
        ]);
        if let Condition::AllActive(items) = &cond3 {
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected AllActive variant");
        }

        // Test NotActive variant (negation)
        let cond4 = Condition::NotActive(alloc::vec![ConditionItem::ModifierActive(0x01),]);
        if let Condition::NotActive(items) = &cond4 {
            assert_eq!(items.len(), 1);
        } else {
            panic!("Expected NotActive variant");
        }
    }
}
