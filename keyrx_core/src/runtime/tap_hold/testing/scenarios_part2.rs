//! Complex integration test scenarios for tap-hold functionality (Part 2).

use super::*;

#[test]
fn test_tap_path_complete_flow() {
    // Complete tap flow: Press → Release(quick) → outputs tap key
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Step 1: Press tap-hold key
    let press_outputs = processor.process_press(KeyCode::CapsLock, 0);

    // No immediate output on press (waiting for release or timeout)
    assert!(press_outputs.is_empty());
    assert!(processor.is_pending(KeyCode::CapsLock));
    assert_eq!(processor.pending_count(), 1);

    // Step 2: Quick release (50ms < 200ms threshold)
    let release_outputs = processor.process_release(KeyCode::CapsLock, 50_000);

    // Should emit tap key (Escape) press + release
    assert_eq!(release_outputs.len(), 2);

    // Verify press event
    match release_outputs[0] {
        TapHoldOutput::KeyEvent {
            key,
            is_press,
            timestamp_us,
        } => {
            assert_eq!(key, KeyCode::Escape);
            assert!(is_press, "First output should be key press");
            assert_eq!(timestamp_us, 50_000);
        }
        _ => panic!("Expected KeyEvent for tap press"),
    }

    // Verify release event
    match release_outputs[1] {
        TapHoldOutput::KeyEvent {
            key,
            is_press,
            timestamp_us,
        } => {
            assert_eq!(key, KeyCode::Escape);
            assert!(!is_press, "Second output should be key release");
            assert_eq!(timestamp_us, 50_000);
        }
        _ => panic!("Expected KeyEvent for tap release"),
    }

    // Step 3: Verify state is cleared
    assert!(!processor.is_pending(KeyCode::CapsLock));
    assert!(!processor.is_hold(KeyCode::CapsLock));
    assert_eq!(processor.pending_count(), 0);
    assert_eq!(processor.hold_count(), 0);
}

#[test]
fn test_tap_path_at_zero_elapsed_time() {
    // Edge case: Release immediately (0μs elapsed)
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 1000);
    let outputs = processor.process_release(KeyCode::CapsLock, 1000); // Same timestamp

    // Should be a tap (0μs < 200ms threshold)
    assert_eq!(outputs.len(), 2);
    match outputs[0] {
        TapHoldOutput::KeyEvent { key, is_press, .. } => {
            assert_eq!(key, KeyCode::Escape);
            assert!(is_press);
        }
        _ => panic!("Expected tap key press"),
    }
}

#[test]
fn test_tap_path_with_different_tap_keys() {
    // Verify different tap keys are emitted correctly
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    // Tab as tap key
    let config = TapHoldConfig::from_ms(KeyCode::Tab, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 0);
    let outputs = processor.process_release(KeyCode::CapsLock, 50_000);

    assert_eq!(outputs.len(), 2);
    match outputs[0] {
        TapHoldOutput::KeyEvent { key, .. } => {
            assert_eq!(key, KeyCode::Tab);
        }
        _ => panic!("Expected Tab key"),
    }
}

// --- Hold Path Tests ---

#[test]
fn test_hold_path_complete_flow() {
    // Complete hold flow: Press → timeout → Hold active → Release → deactivate
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Step 1: Press tap-hold key at t=0
    let press_outputs = processor.process_press(KeyCode::CapsLock, 0);
    assert!(press_outputs.is_empty());
    assert!(processor.is_pending(KeyCode::CapsLock));

    // Step 2: Check timeout before threshold (t=150ms)
    let early_timeout_outputs = processor.check_timeouts(150_000);
    assert!(early_timeout_outputs.is_empty());
    assert!(processor.is_pending(KeyCode::CapsLock)); // Still pending

    // Step 3: Check timeout after threshold (t=250ms)
    let timeout_outputs = processor.check_timeouts(250_000);
    assert_eq!(timeout_outputs.len(), 1);

    // Verify modifier activation
    match timeout_outputs[0] {
        TapHoldOutput::ActivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0);
        }
        _ => panic!("Expected modifier activation"),
    }

    // Verify state transition to Hold
    assert!(processor.is_hold(KeyCode::CapsLock));
    assert!(!processor.is_pending(KeyCode::CapsLock));
    assert_eq!(processor.hold_count(), 1);

    // Step 4: Release the held key
    let release_outputs = processor.process_release(KeyCode::CapsLock, 300_000);
    assert_eq!(release_outputs.len(), 1);

    // Verify modifier deactivation
    match release_outputs[0] {
        TapHoldOutput::DeactivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0);
        }
        _ => panic!("Expected modifier deactivation"),
    }

    // Step 5: Verify state is fully cleared
    assert!(!processor.is_hold(KeyCode::CapsLock));
    assert!(!processor.is_pending(KeyCode::CapsLock));
    assert_eq!(processor.pending_count(), 0);
    assert_eq!(processor.hold_count(), 0);
}

#[test]
fn test_hold_path_with_different_modifiers() {
    // Verify different modifier IDs are activated/deactivated correctly
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    // Use modifier ID 5
    let config = TapHoldConfig::from_ms(KeyCode::Escape, 5, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 0);
    let timeout_outputs = processor.check_timeouts(250_000);

    assert_eq!(timeout_outputs.len(), 1);
    match timeout_outputs[0] {
        TapHoldOutput::ActivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 5);
        }
        _ => panic!("Expected modifier 5 activation"),
    }

    let release_outputs = processor.process_release(KeyCode::CapsLock, 300_000);
    match release_outputs[0] {
        TapHoldOutput::DeactivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 5);
        }
        _ => panic!("Expected modifier 5 deactivation"),
    }
}

#[test]
fn test_hold_path_multiple_timeout_checks() {
    // Verify timeout only triggers once
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // First timeout check - should trigger
    let outputs1 = processor.check_timeouts(250_000);
    assert_eq!(outputs1.len(), 1);
    assert!(processor.is_hold(KeyCode::CapsLock));

    // Second timeout check - should NOT trigger again
    let outputs2 = processor.check_timeouts(300_000);
    assert!(outputs2.is_empty());
    assert!(processor.is_hold(KeyCode::CapsLock)); // Still in hold

    // Third timeout check - still nothing
    let outputs3 = processor.check_timeouts(400_000);
    assert!(outputs3.is_empty());
}

// --- Threshold Edge Cases ---

#[test]
fn test_edge_case_threshold_minus_one_microsecond() {
    // Release at threshold - 1μs (199,999μs): should be TAP
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 0);
    let outputs = processor.process_release(KeyCode::CapsLock, 199_999);

    // 199,999μs < 200,000μs (threshold) → TAP
    assert_eq!(outputs.len(), 2);
    match outputs[0] {
        TapHoldOutput::KeyEvent { key, is_press, .. } => {
            assert_eq!(key, KeyCode::Escape);
            assert!(is_press, "Should emit tap key press");
        }
        _ => panic!("Expected tap key press at threshold-1μs"),
    }
}

#[test]
fn test_edge_case_exact_threshold() {
    // Release at exactly threshold (200,000μs): should be HOLD
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 0);
    let outputs = processor.process_release(KeyCode::CapsLock, 200_000);

    // 200,000μs >= 200,000μs (threshold) → HOLD (activate + deactivate)
    assert_eq!(outputs.len(), 2);
    match outputs[0] {
        TapHoldOutput::ActivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0);
        }
        _ => panic!("Expected modifier activation at exact threshold"),
    }
    match outputs[1] {
        TapHoldOutput::DeactivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0);
        }
        _ => panic!("Expected modifier deactivation at exact threshold"),
    }
}

#[test]
fn test_edge_case_threshold_plus_one_microsecond() {
    // Release at threshold + 1μs (200,001μs): should be HOLD
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 0);
    let outputs = processor.process_release(KeyCode::CapsLock, 200_001);

    // 200,001μs >= 200,000μs (threshold) → HOLD
    assert_eq!(outputs.len(), 2);
    match outputs[0] {
        TapHoldOutput::ActivateModifier { .. } => {}
        _ => panic!("Expected modifier activation at threshold+1μs"),
    }
}

#[test]
fn test_edge_case_timeout_check_at_exact_threshold() {
    // Check timeout at exactly the threshold (200,000μs): should trigger hold
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // Check at exact threshold
    let outputs = processor.check_timeouts(200_000);

    // 200,000μs >= 200,000μs → should trigger hold
    assert_eq!(outputs.len(), 1);
    match outputs[0] {
        TapHoldOutput::ActivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0);
        }
        _ => panic!("Expected modifier activation at exact threshold"),
    }
    assert!(processor.is_hold(KeyCode::CapsLock));
}

#[test]
fn test_edge_case_timeout_check_at_threshold_minus_one() {
    // Check timeout at threshold - 1μs (199,999μs): should NOT trigger
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // Check at threshold - 1μs
    let outputs = processor.check_timeouts(199_999);

    // 199,999μs < 200,000μs → should NOT trigger
    assert!(outputs.is_empty());
    assert!(processor.is_pending(KeyCode::CapsLock));
}

#[test]
fn test_edge_case_timeout_check_at_threshold_plus_one() {
    // Check timeout at threshold + 1μs (200,001μs): should trigger hold
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // Check at threshold + 1μs
    let outputs = processor.check_timeouts(200_001);

    // 200,001μs >= 200,000μs → should trigger
    assert_eq!(outputs.len(), 1);
    assert!(processor.is_hold(KeyCode::CapsLock));
}

#[test]
fn test_edge_case_zero_threshold() {
    // Edge case: 0ms threshold - any press should immediately be hold
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 0); // 0ms threshold
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // Check timeout immediately - should trigger because 0 >= 0
    let outputs = processor.check_timeouts(0);
    assert_eq!(outputs.len(), 1);
    assert!(processor.is_hold(KeyCode::CapsLock));
}

#[test]
fn test_edge_case_very_long_hold() {
    // Hold for a very long time (10 seconds)
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 0);
    let _ = processor.check_timeouts(10_000_000); // 10 seconds

    assert!(processor.is_hold(KeyCode::CapsLock));

    // Release after very long hold
    let outputs = processor.process_release(KeyCode::CapsLock, 10_000_000);
    assert_eq!(outputs.len(), 1);
    match outputs[0] {
        TapHoldOutput::DeactivateModifier { modifier_id } => {
            assert_eq!(modifier_id, 0);
        }
        _ => panic!("Expected modifier deactivation after long hold"),
    }
}

#[test]
fn test_edge_case_max_threshold() {
    // Maximum threshold: u16::MAX ms = 65,535ms
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, u16::MAX);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    let _ = processor.process_press(KeyCode::CapsLock, 0);

    // Check at 65 seconds - should NOT trigger yet
    let outputs = processor.check_timeouts(65_000_000);
    assert!(outputs.is_empty());
    assert!(processor.is_pending(KeyCode::CapsLock));

    // Check at 66 seconds (above 65.535s) - should trigger
    let outputs = processor.check_timeouts(66_000_000);
    assert_eq!(outputs.len(), 1);
    assert!(processor.is_hold(KeyCode::CapsLock));
}

#[test]
fn test_edge_case_nonzero_press_time() {
    // Press at non-zero time (simulating real-world usage)
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Press at t=1,000,000μs (1 second into session)
    let _ = processor.process_press(KeyCode::CapsLock, 1_000_000);

    // Tap: release at t=1,050,000μs (50ms elapsed)
    let outputs = processor.process_release(KeyCode::CapsLock, 1_050_000);
    assert_eq!(outputs.len(), 2);
    match outputs[0] {
        TapHoldOutput::KeyEvent { key, .. } => {
            assert_eq!(key, KeyCode::Escape);
        }
        _ => panic!("Expected tap key"),
    }
}

#[test]
fn test_edge_case_nonzero_press_time_hold() {
    // Hold with non-zero press time
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Press at t=5,000,000μs (5 seconds into session)
    let _ = processor.process_press(KeyCode::CapsLock, 5_000_000);

    // Check at t=5,199,999μs (199,999μs elapsed) - should NOT trigger
    let outputs = processor.check_timeouts(5_199_999);
    assert!(outputs.is_empty());

    // Check at t=5,200,000μs (exactly 200ms elapsed) - should trigger
    let outputs = processor.check_timeouts(5_200_000);
    assert_eq!(outputs.len(), 1);
    assert!(processor.is_hold(KeyCode::CapsLock));
}

// --- State Machine Integrity Tests ---

#[test]
fn test_state_machine_idle_to_pending_to_tap() {
    // Full state trace: Idle → Pending → (release quick) → Idle
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Initial state: nothing tracked
    assert!(!processor.is_pending(KeyCode::CapsLock));
    assert!(!processor.is_hold(KeyCode::CapsLock));

    // Press → Pending
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    assert!(processor.is_pending(KeyCode::CapsLock));
    assert!(!processor.is_hold(KeyCode::CapsLock));

    // Quick release → back to nothing (tap occurred)
    let _ = processor.process_release(KeyCode::CapsLock, 50_000);
    assert!(!processor.is_pending(KeyCode::CapsLock));
    assert!(!processor.is_hold(KeyCode::CapsLock));
}

#[test]
fn test_state_machine_idle_to_pending_to_hold_to_idle() {
    // Full state trace: Idle → Pending → Hold → Idle
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Initial state
    assert!(!processor.is_pending(KeyCode::CapsLock));
    assert!(!processor.is_hold(KeyCode::CapsLock));

    // Press → Pending
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    assert!(processor.is_pending(KeyCode::CapsLock));
    assert!(!processor.is_hold(KeyCode::CapsLock));

    // Timeout → Hold
    let _ = processor.check_timeouts(250_000);
    assert!(!processor.is_pending(KeyCode::CapsLock));
    assert!(processor.is_hold(KeyCode::CapsLock));

    // Release → Idle
    let _ = processor.process_release(KeyCode::CapsLock, 300_000);
    assert!(!processor.is_pending(KeyCode::CapsLock));
    assert!(!processor.is_hold(KeyCode::CapsLock));
}

#[test]
fn test_state_machine_rapid_tap_sequence() {
    // Multiple rapid taps in sequence
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    for i in 0..5 {
        let base_time = i * 100_000; // 100ms apart

        // Press
        let _ = processor.process_press(KeyCode::CapsLock, base_time);
        assert!(processor.is_pending(KeyCode::CapsLock));

        // Quick release (50ms later)
        let outputs = processor.process_release(KeyCode::CapsLock, base_time + 50_000);
        assert_eq!(outputs.len(), 2);
        assert!(!processor.is_pending(KeyCode::CapsLock));
    }
}

#[test]
fn test_state_machine_alternating_tap_hold() {
    // Alternate between taps and holds
    let mut processor: TapHoldProcessor<8> = TapHoldProcessor::new();

    let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    processor.register_tap_hold(KeyCode::CapsLock, config);

    // Tap #1
    let _ = processor.process_press(KeyCode::CapsLock, 0);
    let outputs = processor.process_release(KeyCode::CapsLock, 50_000);
    assert_eq!(outputs.len(), 2); // tap key press + release

    // Hold #1
    let _ = processor.process_press(KeyCode::CapsLock, 100_000);
    let outputs = processor.check_timeouts(350_000);
    assert_eq!(outputs.len(), 1); // modifier activation
    let outputs = processor.process_release(KeyCode::CapsLock, 400_000);
    assert_eq!(outputs.len(), 1); // modifier deactivation

    // Tap #2
    let _ = processor.process_press(KeyCode::CapsLock, 500_000);
    let outputs = processor.process_release(KeyCode::CapsLock, 550_000);
    assert_eq!(outputs.len(), 2); // tap key press + release

    // Final state check
    assert!(!processor.is_pending(KeyCode::CapsLock));
    assert!(!processor.is_hold(KeyCode::CapsLock));
}
