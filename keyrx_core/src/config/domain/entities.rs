//! Domain entities for Configuration domain
//!
//! Entities have unique identity and lifecycle.

use alloc::string::String;
use alloc::vec::Vec;

use crate::config::KeyCode;

/// Modifier entity with unique identity
///
/// Represents a custom modifier (MD_00-MD_FE) with identity and lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierEntity {
    /// Unique modifier ID (0-254)
    id: u8,
    /// Human-readable name
    name: String,
    /// Description of this modifier's purpose
    description: String,
    /// Whether this modifier is currently active
    active: bool,
}

impl ModifierEntity {
    /// Creates a new Modifier entity
    pub fn new(id: u8, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
            active: false,
        }
    }

    /// Gets the modifier ID
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Gets the modifier name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Checks if this modifier is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Activates this modifier
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Deactivates this modifier
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Updates the name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Updates the description
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }
}

/// Lock entity with unique identity
///
/// Represents a custom lock (LK_00-LK_FE) with identity and lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockEntity {
    /// Unique lock ID (0-254)
    id: u8,
    /// Human-readable name
    name: String,
    /// Description of this lock's purpose
    description: String,
    /// Whether this lock is currently engaged
    engaged: bool,
}

impl LockEntity {
    /// Creates a new Lock entity
    pub fn new(id: u8, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
            engaged: false,
        }
    }

    /// Gets the lock ID
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Gets the lock name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Checks if this lock is engaged
    pub fn is_engaged(&self) -> bool {
        self.engaged
    }

    /// Engages this lock
    pub fn engage(&mut self) {
        self.engaged = true;
    }

    /// Disengages this lock
    pub fn disengage(&mut self) {
        self.engaged = false;
    }

    /// Toggles this lock
    pub fn toggle(&mut self) {
        self.engaged = !self.engaged;
    }

    /// Updates the name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Updates the description
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }
}

/// Macro entity with unique identity
///
/// Represents a macro sequence with identity and lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroEntity {
    /// Unique macro name
    name: String,
    /// Sequence of key codes to execute
    sequence: Vec<KeyCode>,
    /// Delay between keys in milliseconds
    delay_ms: u16,
    /// Whether this macro is enabled
    enabled: bool,
    /// Number of times this macro has been executed
    execution_count: u64,
}

impl MacroEntity {
    /// Creates a new Macro entity
    pub fn new(name: String, sequence: Vec<KeyCode>, delay_ms: u16) -> Self {
        Self {
            name,
            sequence,
            delay_ms,
            enabled: true,
            execution_count: 0,
        }
    }

    /// Gets the macro name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the sequence
    pub fn sequence(&self) -> &[KeyCode] {
        &self.sequence
    }

    /// Gets the delay between keys
    pub fn delay_ms(&self) -> u16 {
        self.delay_ms
    }

    /// Checks if this macro is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Gets the execution count
    pub fn execution_count(&self) -> u64 {
        self.execution_count
    }

    /// Enables this macro
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disables this macro
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Records an execution
    pub fn record_execution(&mut self) {
        self.execution_count += 1;
    }

    /// Resets execution count
    pub fn reset_execution_count(&mut self) {
        self.execution_count = 0;
    }

    /// Updates the sequence
    pub fn set_sequence(&mut self, sequence: Vec<KeyCode>) {
        self.sequence = sequence;
    }

    /// Updates the delay
    pub fn set_delay_ms(&mut self, delay_ms: u16) {
        self.delay_ms = delay_ms;
    }

    /// Gets the estimated execution time in milliseconds
    pub fn estimated_duration_ms(&self) -> u64 {
        (self.sequence.len() as u64) * (self.delay_ms as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifier_entity_creation() {
        let modifier = ModifierEntity::new(1, "Hyper".into(), "Hyper modifier".into());

        assert_eq!(modifier.id(), 1);
        assert_eq!(modifier.name(), "Hyper");
        assert_eq!(modifier.description(), "Hyper modifier");
        assert!(!modifier.is_active());
    }

    #[test]
    fn test_modifier_entity_activation() {
        let mut modifier = ModifierEntity::new(1, "Hyper".into(), "Hyper modifier".into());

        assert!(!modifier.is_active());
        modifier.activate();
        assert!(modifier.is_active());
        modifier.deactivate();
        assert!(!modifier.is_active());
    }

    #[test]
    fn test_modifier_entity_updates() {
        let mut modifier = ModifierEntity::new(1, "Hyper".into(), "Hyper modifier".into());

        modifier.set_name("Super".into());
        assert_eq!(modifier.name(), "Super");

        modifier.set_description("Super modifier".into());
        assert_eq!(modifier.description(), "Super modifier");
    }

    #[test]
    fn test_lock_entity_creation() {
        let lock = LockEntity::new(1, "VimMode".into(), "Vim mode lock".into());

        assert_eq!(lock.id(), 1);
        assert_eq!(lock.name(), "VimMode");
        assert_eq!(lock.description(), "Vim mode lock");
        assert!(!lock.is_engaged());
    }

    #[test]
    fn test_lock_entity_toggle() {
        let mut lock = LockEntity::new(1, "VimMode".into(), "Vim mode lock".into());

        assert!(!lock.is_engaged());
        lock.toggle();
        assert!(lock.is_engaged());
        lock.toggle();
        assert!(!lock.is_engaged());
    }

    #[test]
    fn test_lock_entity_engage_disengage() {
        let mut lock = LockEntity::new(1, "VimMode".into(), "Vim mode lock".into());

        lock.engage();
        assert!(lock.is_engaged());
        lock.disengage();
        assert!(!lock.is_engaged());
    }

    #[test]
    fn test_macro_entity_creation() {
        let macro_entity = MacroEntity::new(
            "email".into(),
            alloc::vec![KeyCode::T, KeyCode::E, KeyCode::S, KeyCode::T],
            50,
        );

        assert_eq!(macro_entity.name(), "email");
        assert_eq!(macro_entity.sequence().len(), 4);
        assert_eq!(macro_entity.delay_ms(), 50);
        assert!(macro_entity.is_enabled());
        assert_eq!(macro_entity.execution_count(), 0);
    }

    #[test]
    fn test_macro_entity_execution_tracking() {
        let mut macro_entity = MacroEntity::new(
            "email".into(),
            alloc::vec![KeyCode::T, KeyCode::E, KeyCode::S, KeyCode::T],
            50,
        );

        assert_eq!(macro_entity.execution_count(), 0);

        macro_entity.record_execution();
        assert_eq!(macro_entity.execution_count(), 1);

        macro_entity.record_execution();
        assert_eq!(macro_entity.execution_count(), 2);

        macro_entity.reset_execution_count();
        assert_eq!(macro_entity.execution_count(), 0);
    }

    #[test]
    fn test_macro_entity_duration_calculation() {
        let macro_entity = MacroEntity::new(
            "test".into(),
            alloc::vec![KeyCode::A, KeyCode::B, KeyCode::C],
            100,
        );

        // 3 keys * 100ms = 300ms
        assert_eq!(macro_entity.estimated_duration_ms(), 300);
    }

    #[test]
    fn test_macro_entity_enable_disable() {
        let mut macro_entity = MacroEntity::new("test".into(), alloc::vec![KeyCode::A], 50);

        assert!(macro_entity.is_enabled());
        macro_entity.disable();
        assert!(!macro_entity.is_enabled());
        macro_entity.enable();
        assert!(macro_entity.is_enabled());
    }

    #[test]
    fn test_macro_entity_updates() {
        let mut macro_entity = MacroEntity::new("test".into(), alloc::vec![KeyCode::A], 50);

        macro_entity.set_sequence(alloc::vec![KeyCode::B, KeyCode::C]);
        assert_eq!(macro_entity.sequence().len(), 2);

        macro_entity.set_delay_ms(100);
        assert_eq!(macro_entity.delay_ms(), 100);
        assert_eq!(macro_entity.estimated_duration_ms(), 200);
    }
}
