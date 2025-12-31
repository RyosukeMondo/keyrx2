//! Complex integration test scenarios for tap-hold functionality (Part 1).

use super::*;

// --- Test: Interrupted tap-hold confirms Hold immediately ---

#[test]
fn test_permissive_hold_immediate_hold_confirmation() {
    // Verify that permissive hold transitions to Hold state BEFORE returning
    // This is critical for ensuring modifier is active before interrupting key
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Press CapsLock - enters Pending
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    assert!(processor.is_pending(KeyCode::CapsLock));
    assert!(!processor.is_hold(KeyCode::CapsLock));

    // Press another key - should IMMEDIATELY confirm Hold
    let outputs = processor.process_other_key_press(KeyCode::A);

    // State transition should happen BEFORE outputs returned
    // This ensures caller can rely on modifier being active
    assert!(
        processor.is_hold(KeyCode::CapsLock),
        "Should be in Hold state immediately"
    );
    assert!(
        !processor.is_pending(KeyCode::CapsLock),
        "Should no longer be Pending"
    );
    assert_eq!(outputs.len(), 1);
}

#[test]
fn test_permissive_hold_immediate_transition_with_timing() {
    // Test that permissive hold ignores timing - any interrupt triggers Hold
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    // Use a 500ms threshold
    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 500);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Press at t=0
    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // Interrupt at t=1μs (way before 500ms threshold)
    // Should still transition to Hold immediately
    let outputs = processor.process_other_key_press(KeyCode::A);

    assert!(processor.is_hold(KeyCode::CapsLock));
    assert_eq!(outputs.len(), 1);

    match outputs[0] {
        TapHoldOutput::ActivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0);
        }
        _ => panic!("Expected ActivateModifier"),
    }
}

// --- Test: Modifier active before interrupted key processed ---

#[test]
fn test_permissive_hold_modifier_active_before_output_returned() {
    // The modifier activation output is returned so caller can apply it
    // before processing the interrupting key
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // Before interrupt - no modifier active
    assert_eq!(processor.hold_count(), 0);

    let outputs = processor.process_other_key_press(KeyCode::A);

    // After interrupt - modifier should be active
    assert_eq!(processor.hold_count(), 1);

    // Output contains the activation instruction
    assert_eq!(outputs.len(), 1);
    assert!(matches!(
        outputs[0],
        TapHoldOutput::ActivateModifier { modifier_id: 0 }
    ));
}

#[test]
fn test_permissive_hold_modifier_order_with_multiple_keys() {
    // When multiple modifiers activate, they should all be in outputs
    // and all should be in Hold state
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config_caps = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200); // modifier 0 (Ctrl)
    let config_tab = TapHoldConfig::from_ms(KeyCode::Tab, 1, 200); // modifier 1 (Alt)
    let config_shift = TapHoldConfig::from_ms(KeyCode::A, 2, 200); // modifier 2 (Shift)

    processor.register_tap_hold(KeyCode::CapsLock, config_caps);
    processor.register_tap_hold(KeyCode::Tab, config_tab);
    processor.register_tap_hold(KeyCode::LShift, config_shift);

    // Press all three tap-hold keys
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    let _ = processor.process_press(KeyCode::Tab, 10_000);
    let _ = processor.process_press(KeyCode::LShift, 20_000);

    assert_eq!(processor.pending_count(), 3);
    assert_eq!(processor.hold_count(), 0);

    // Interrupt with regular key
    let outputs = processor.process_other_key_press(KeyCode::B);

    // All three should now be in Hold
    assert_eq!(processor.hold_count(), 3);
    assert_eq!(processor.pending_count(), 0);

    // All three modifiers should be activated
    assert_eq!(outputs.len(), 3);

    // Collect modifier IDs
    let mut modifier_ids: Vec<u8> = outputs
        .iter()
        .filter_map(|o| match o {
            TapHoldOutput::ActivateModifier { modifier_id } => Some(*modifier_id),
            _ => None,
        })
        .collect();
    modifier_ids.sort();

    assert_eq!(modifier_ids, vec![0, 1, 2]);
}

// --- Test: Multiple concurrent tap-holds with interruption ---

#[test]
fn test_permissive_hold_concurrent_staggered_timing() {
    // Multiple tap-holds pressed at different times, then interrupted
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config1 = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    let config2 = TapHoldConfig::from_ms(KeyCode::Tab, 1, 300); // Different threshold

    processor.register_tap_hold(KeyCode::CapsLock, config1);
    processor.register_tap_hold(KeyCode::Tab, config2);

    // CapsLock pressed at t=0
    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // Tab pressed at t=100ms
    let _ = processor.process_press(KeyCode::Tab, 100_000);

    // Both should be Pending
    assert!(processor.is_pending(KeyCode::CapsLock));
    assert!(processor.is_pending(KeyCode::Tab));

    // Interrupt at t=150ms (before both thresholds)
    let outputs = processor.process_other_key_press(KeyCode::A);

    // Both should transition to Hold regardless of their individual thresholds
    assert!(processor.is_hold(KeyCode::CapsLock));
    assert!(processor.is_hold(KeyCode::Tab));
    assert_eq!(outputs.len(), 2);
}

#[test]
fn test_permissive_hold_one_timed_out_one_pending() {
    // One key has timed out (already Hold), one still Pending
    // Only Pending should be affected by permissive hold
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config1 = TapHoldConfig::from_ms(KeyCode::Escape, 0, 100); // 100ms threshold
    let config2 = TapHoldConfig::from_ms(KeyCode::Tab, 1, 300); // 300ms threshold

    processor.register_tap_hold(KeyCode::CapsLock, config1);
    processor.register_tap_hold(KeyCode::Tab, config2);

    // Press both
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    let _ = processor.process_press(KeyCode::Tab, 50_000);

    // At t=150ms: CapsLock times out (150ms > 100ms), Tab still pending (100ms < 300ms)
    let timeouts = processor.check_timeouts(150_000);
    assert_eq!(timeouts.len(), 1);
    assert!(processor.is_hold(KeyCode::CapsLock));
    assert!(processor.is_pending(KeyCode::Tab));

    // Now interrupt with another key
    let outputs = processor.process_other_key_press(KeyCode::A);

    // Only Tab should transition (CapsLock already in Hold)
    assert_eq!(outputs.len(), 1);
    match outputs[0] {
        TapHoldOutput::ActivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 1, "Only Tab's modifier should activate");
        }
        _ => panic!("Expected ActivateModifier"),
    }

    // Both should now be in Hold
    assert!(processor.is_hold(KeyCode::CapsLock));
    assert!(processor.is_hold(KeyCode::Tab));
}

#[test]
fn test_permissive_hold_rapid_key_presses() {
    // Rapid sequence: tap-hold press → immediate interrupt → another interrupt
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Press tap-hold key
    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // First interrupt - transitions to Hold
    let outputs1 = processor.process_other_key_press(KeyCode::A);
    assert_eq!(outputs1.len(), 1);
    assert!(processor.is_hold(KeyCode::CapsLock));

    // Second "interrupt" - already in Hold, no change
    let outputs2 = processor.process_other_key_press(KeyCode::B);
    assert!(outputs2.is_empty());
    assert!(processor.is_hold(KeyCode::CapsLock));

    // Third "interrupt" - still no change
    let outputs3 = processor.process_other_key_press(KeyCode::C);
    assert!(outputs3.is_empty());
}

// --- Registry-level Permissive Hold tests ---

#[test]
fn test_registry_permissive_hold_empty_registry() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    // Empty registry - no panic, just empty results
    let results = registry.trigger_permissive_hold();
    assert!(results.is_empty());
}

#[test]
fn test_registry_permissive_hold_all_already_hold() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    // Add two states
    let mut state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
    let mut state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 0);

    // Manually put both in Hold state
    state1.transition_to_hold();
    state2.transition_to_hold();

    registry.add(state1);
    registry.add(state2);

    // Trigger permissive hold - none should be affected
    let results = registry.trigger_permissive_hold();
    assert!(results.is_empty());
}

#[test]
fn test_registry_permissive_hold_mixed_phases() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    // State 1: Pending
    let state1 = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
    // State 2: Hold
    let mut state2 = make_pending_state(KeyCode::Tab, KeyCode::Tab, 1, 0);
    state2.transition_to_hold();
    // State 3: Pending
    let state3 = make_pending_state(KeyCode::LShift, KeyCode::A, 2, 0);

    registry.add(state1);
    registry.add(state2);
    registry.add(state3);

    let results = registry.trigger_permissive_hold();

    // Only Pending states should be in results
    assert_eq!(results.len(), 2);

    let modifier_ids: Vec<u8> = results.iter().map(|r| r.hold_modifier).collect();
    assert!(modifier_ids.contains(&0)); // CapsLock's modifier
    assert!(modifier_ids.contains(&2)); // LeftShift's modifier
    assert!(!modifier_ids.contains(&1)); // Tab was already Hold
}

#[test]
fn test_registry_permissive_hold_verifies_state_change() {
    let mut registry: PendingKeyRegistry<8> = PendingKeyRegistry::new();

    let state = make_pending_state(KeyCode::CapsLock, KeyCode::Escape, 0, 0);
    registry.add(state);

    // Before trigger
    assert!(registry
        .get(KeyCode::CapsLock)
        .unwrap()
        .phase()
        .is_pending());

    let _ = registry.trigger_permissive_hold();

    // After trigger - state should be Hold
    assert!(registry.get(KeyCode::CapsLock).unwrap().phase().is_hold());
}

// --- Edge case tests ---

#[test]
fn test_permissive_hold_then_timeout_check_no_duplicate() {
    // After permissive hold triggers, timeout check shouldn't re-trigger
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // Permissive hold at t=50ms
    let outputs1 = processor.process_other_key_press(KeyCode::A);
    assert_eq!(outputs1.len(), 1);

    // Timeout check at t=300ms (past threshold) - should not re-trigger
    let timeouts = processor.check_timeouts(300_000);
    assert!(timeouts.is_empty()); // Already in Hold, no new timeouts

    // Verify still in Hold
    assert!(processor.is_hold(KeyCode::CapsLock));
}

#[test]
fn test_permissive_hold_self_interrupt_ignored() {
    // Pressing the same tap-hold key again shouldn't trigger permissive hold
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // Try to "interrupt" with the same key
    let outputs = processor.process_other_key_press(KeyCode::CapsLock);

    // Should be ignored - key is in pending registry
    assert!(outputs.is_empty());
    assert!(processor.is_pending(KeyCode::CapsLock));
}

#[test]
fn test_permissive_hold_release_after_permissive_triggers_deactivation() {
    // Full flow: press → permissive hold → release
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Press
    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // Permissive hold interrupt
    let ph_outputs = processor.process_other_key_press(KeyCode::A);
    assert_eq!(ph_outputs.len(), 1);
    assert!(matches!(
        ph_outputs[0],
        TapHoldOutput::ActivateModifier { modifier_id: 0 }
    ));

    // Release tap-hold key
    let release_outputs = processor.process_release(KeyCode::CapsLock, 100_000);

    // Should deactivate modifier (not emit tap key)
    assert_eq!(release_outputs.len(), 1);
    assert!(matches!(
        release_outputs[0],
        TapHoldOutput::DeactivateModifier { modifier_id: 0 }
    ));

    // Should no longer be tracked
    assert!(!processor.is_hold(KeyCode::CapsLock));
    assert!(!processor.is_pending(KeyCode::CapsLock));
}

#[test]
fn test_permissive_hold_realistic_ctrl_shift_a() {
    // Realistic scenario: Ctrl+Shift+A using CapsLock(Ctrl) and Tab(Shift)
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let ctrl_config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200); // Ctrl
    let shift_config = TapHoldConfig::from_ms(KeyCode::Tab, 1, 200); // Shift

    processor.register_tap_hold(KeyCode::CapsLock, ctrl_config);
    processor.register_tap_hold(KeyCode::Tab, shift_config);

    // User presses CapsLock (intending Ctrl)
    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // User presses Tab (intending Shift)
    // Note: This should NOT trigger permissive hold for CapsLock via process_other_key_press
    // because Tab is a registered tap-hold key
    let _ = processor.process_press(KeyCode::Tab, 30_000);

    // Both should be pending
    assert!(processor.is_pending(KeyCode::CapsLock));
    assert!(processor.is_pending(KeyCode::Tab));

    // User presses 'A' - this triggers permissive hold for BOTH
    let outputs = processor.process_other_key_press(KeyCode::A);

    // Both Ctrl and Shift should activate
    assert_eq!(outputs.len(), 2);
    assert!(processor.is_hold(KeyCode::CapsLock));
    assert!(processor.is_hold(KeyCode::Tab));

    // User releases 'A' (handled by caller)

    // User releases Tab (Shift)
    let tab_release = processor.process_release(KeyCode::Tab, 100_000);
    assert_eq!(tab_release.len(), 1);
    assert!(matches!(
        tab_release[0],
        TapHoldOutput::DeactivateModifier { modifier_id: 1 }
    ));

    // User releases CapsLock (Ctrl)
    let caps_release = processor.process_release(KeyCode::CapsLock, 150_000);
    assert_eq!(caps_release.len(), 1);
    assert!(matches!(
        caps_release[0],
        TapHoldOutput::DeactivateModifier { modifier_id: 0 }
    ));

    // All cleaned up
    assert_eq!(processor.hold_count(), 0);
    assert_eq!(processor.pending_count(), 0);
}

#[test]
fn test_permissive_hold_different_modifier_ids() {
    // Test with non-sequential modifier IDs
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config1 = TapHoldConfig::from_ms(KeyCode::Escape, 5, 200); // modifier 5
    let config2 = TapHoldConfig::from_ms(KeyCode::Tab, 10, 200); // modifier 10
    let config3 = TapHoldConfig::from_ms(KeyCode::A, 255, 200); // modifier 255 (max)

    processor.register_tap_hold(KeyCode::CapsLock, config1);
    processor.register_tap_hold(KeyCode::Tab, config2);
    processor.register_tap_hold(KeyCode::LShift, config3);

    let _ = processor.process_press(KeyCode::CapsLock, 0);
    let _ = processor.process_press(KeyCode::Tab, 0);
    let _ = processor.process_press(KeyCode::LShift, 0);

    let outputs = processor.process_other_key_press(KeyCode::B);

    assert_eq!(outputs.len(), 3);

    let mut modifier_ids: Vec<u8> = outputs
        .iter()
        .filter_map(|o| match o {
            TapHoldOutput::ActivateModifier { modifier_id } => Some(*modifier_id),
            _ => None,
        })
        .collect();
    modifier_ids.sort();

    assert_eq!(modifier_ids, vec![5, 10, 255]);
}

#[test]
fn test_permissive_hold_max_concurrent_keys() {
    // Test with processor at capacity
    let mut processor: TapHoldProcessor<4> = TapHoldProcessor::new();

    // Register 4 tap-hold keys (max capacity)
    for i in 0u8..4 {
        let config = TapHoldConfig::from_ms(KeyCode::A, i, 200);
        // Use different keys for each registration
        let key = match i {
            0 => KeyCode::CapsLock,
            1 => KeyCode::Tab,
            2 => KeyCode::LShift,
            _ => KeyCode::LCtrl,
        };
        processor.register_tap_hold(key, config);
    }

    // Press all 4
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    let _ = processor.process_press(KeyCode::Tab, 0);
    let _ = processor.process_press(KeyCode::LShift, 0);
    let _ = processor.process_press(KeyCode::LCtrl, 0);

    assert_eq!(processor.pending_count(), 4);

    // Interrupt - all should transition
    let outputs = processor.process_other_key_press(KeyCode::A);

    assert_eq!(outputs.len(), 4);
    assert_eq!(processor.hold_count(), 4);
    assert_eq!(processor.pending_count(), 0);
}

// =======================================================================
// Task 10: Comprehensive State Machine Tests
// =======================================================================

// --- Tap Path Tests ---
