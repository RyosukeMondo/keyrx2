//! Virtual E2E Tests for keyrx daemon.
//!
//! These tests use virtual input devices (uinput) to test the complete
//! keyboard remapping pipeline without requiring physical hardware.
//!
//! # Running These Tests
//!
//! These tests require:
//! - Linux with uinput module loaded (`sudo modprobe uinput`)
//! - Write access to `/dev/uinput` (usually requires root or uinput group)
//! - The keyrx_daemon binary built
//!
//! Run with:
//! ```bash
//! sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests -- --ignored
//! ```
//!
//! Or run all E2E tests:
//! ```bash
//! sudo cargo test -p keyrx_daemon --features linux -- --ignored
//! ```

#![cfg(all(target_os = "linux", feature = "linux"))]

mod e2e_harness;

use std::time::Duration;

use e2e_harness::{E2EConfig, E2EHarness, TestEvents};
use keyrx_core::config::KeyCode;
use keyrx_core::runtime::KeyEvent;

// ============================================================================
// Simple Remap Tests - Requirement 5.1
// ============================================================================

/// Test simple A → B remapping (press event).
///
/// Verifies that when A is pressed, B is output instead.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_simple_remap_press -- --ignored"]
fn test_simple_remap_press() {
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject Press(A)
    let input = TestEvents::press(KeyCode::A);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(100))
        .expect("Failed to inject and capture");

    // Expect Press(B)
    let expected = TestEvents::press(KeyCode::B);
    harness
        .verify(&captured, &expected)
        .expect("Press A should produce Press B");
}

/// Test simple A → B remapping (release event).
///
/// Verifies that when A is released, B release is output instead.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_simple_remap_release -- --ignored"]
fn test_simple_remap_release() {
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // First press A to establish state
    harness
        .inject(&TestEvents::press(KeyCode::A))
        .expect("Failed to inject press");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Now inject Release(A)
    let input = TestEvents::release(KeyCode::A);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(100))
        .expect("Failed to inject and capture");

    // Expect Release(B)
    let expected = TestEvents::release(KeyCode::B);
    harness
        .verify(&captured, &expected)
        .expect("Release A should produce Release B");
}

/// Test simple A → B remapping (complete key tap).
///
/// Verifies that a complete tap (press + release) of A produces
/// a complete tap of B.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_simple_remap_tap -- --ignored"]
fn test_simple_remap_tap() {
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject tap(A) = [Press(A), Release(A)]
    let input = TestEvents::tap(KeyCode::A);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(100))
        .expect("Failed to inject and capture");

    // Expect tap(B) = [Press(B), Release(B)]
    let expected = TestEvents::tap(KeyCode::B);
    harness
        .verify(&captured, &expected)
        .expect("Tap A should produce Tap B");
}

// ============================================================================
// Multiple Remaps in Sequence Tests
// ============================================================================

/// Test multiple different remaps in the same configuration.
///
/// Verifies that when multiple remaps are configured (A→B, C→D),
/// each key is correctly remapped.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_multiple_remaps_different_keys -- --ignored"]
fn test_multiple_remaps_different_keys() {
    // Configure A→B and C→D
    let config = E2EConfig::simple_remaps(vec![(KeyCode::A, KeyCode::B), (KeyCode::C, KeyCode::D)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Test A→B
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture A");
    harness
        .verify(&captured, &TestEvents::tap(KeyCode::B))
        .expect("Tap A should produce Tap B");

    // Test C→D
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::C), Duration::from_millis(100))
        .expect("Failed to inject and capture C");
    harness
        .verify(&captured, &TestEvents::tap(KeyCode::D))
        .expect("Tap C should produce Tap D");
}

/// Test sequence of same remapped key.
///
/// Verifies that repeatedly pressing the same remapped key works correctly.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_repeated_remap_sequence -- --ignored"]
fn test_repeated_remap_sequence() {
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject A three times
    let input = TestEvents::taps(&[KeyCode::A, KeyCode::A, KeyCode::A]);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(200))
        .expect("Failed to inject and capture");

    // Expect B three times
    let expected = TestEvents::taps(&[KeyCode::B, KeyCode::B, KeyCode::B]);
    harness
        .verify(&captured, &expected)
        .expect("Three taps of A should produce three taps of B");
}

/// Test alternating between remapped keys.
///
/// Verifies that alternating between different remapped keys works correctly.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_alternating_remapped_keys -- --ignored"]
fn test_alternating_remapped_keys() {
    let config = E2EConfig::simple_remaps(vec![(KeyCode::A, KeyCode::B), (KeyCode::C, KeyCode::D)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject A, C, A pattern
    let input = TestEvents::taps(&[KeyCode::A, KeyCode::C, KeyCode::A]);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(200))
        .expect("Failed to inject and capture");

    // Expect B, D, B pattern
    let expected = TestEvents::taps(&[KeyCode::B, KeyCode::D, KeyCode::B]);
    harness
        .verify(&captured, &expected)
        .expect("A, C, A should produce B, D, B");
}

// ============================================================================
// Unmapped Key Passthrough Tests - Requirement 5.6
// ============================================================================

/// Test that unmapped keys pass through unchanged.
///
/// Verifies that when A→B is configured, pressing an unmapped key (C)
/// produces C without modification.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_unmapped_key_passthrough -- --ignored"]
fn test_unmapped_key_passthrough() {
    // Only A→B is configured
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject tap(C) which is not mapped
    let input = TestEvents::tap(KeyCode::C);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(100))
        .expect("Failed to inject and capture");

    // Expect tap(C) unchanged
    let expected = TestEvents::tap(KeyCode::C);
    harness
        .verify(&captured, &expected)
        .expect("Unmapped key C should pass through unchanged");
}

/// Test multiple unmapped keys in sequence.
///
/// Verifies that a sequence of unmapped keys all pass through correctly.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_multiple_unmapped_keys_passthrough -- --ignored"]
fn test_multiple_unmapped_keys_passthrough() {
    // Only A→B is configured
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject sequence of unmapped keys
    let input = TestEvents::taps(&[KeyCode::X, KeyCode::Y, KeyCode::Z]);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(200))
        .expect("Failed to inject and capture");

    // Expect same sequence unchanged
    let expected = TestEvents::taps(&[KeyCode::X, KeyCode::Y, KeyCode::Z]);
    harness
        .verify(&captured, &expected)
        .expect("Unmapped keys X, Y, Z should all pass through unchanged");
}

/// Test mixed remapped and unmapped keys.
///
/// Verifies that remapped keys are transformed while unmapped keys
/// pass through in the same sequence.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_mixed_mapped_unmapped_keys -- --ignored"]
fn test_mixed_mapped_unmapped_keys() {
    // A→B configured, but not C
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject: C (unmapped), A (mapped), C (unmapped)
    let input = TestEvents::taps(&[KeyCode::C, KeyCode::A, KeyCode::C]);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(200))
        .expect("Failed to inject and capture");

    // Expect: C (pass), B (remapped from A), C (pass)
    let expected = TestEvents::taps(&[KeyCode::C, KeyCode::B, KeyCode::C]);
    harness
        .verify(&captured, &expected)
        .expect("C should pass through, A should become B");
}

/// Test special keys passthrough (modifiers, function keys).
///
/// Verifies that special keys like Shift, Ctrl, F-keys pass through
/// when not explicitly mapped.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_special_keys_passthrough -- --ignored"]
fn test_special_keys_passthrough() {
    // Only A→B configured
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Test Escape passthrough
    let captured = harness
        .inject_and_capture(
            &TestEvents::tap(KeyCode::Escape),
            Duration::from_millis(100),
        )
        .expect("Failed to inject Escape");
    harness
        .verify(&captured, &TestEvents::tap(KeyCode::Escape))
        .expect("Escape should pass through");

    // Test F1 passthrough
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::F1), Duration::from_millis(100))
        .expect("Failed to inject F1");
    harness
        .verify(&captured, &TestEvents::tap(KeyCode::F1))
        .expect("F1 should pass through");

    // Test Tab passthrough
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::Tab), Duration::from_millis(100))
        .expect("Failed to inject Tab");
    harness
        .verify(&captured, &TestEvents::tap(KeyCode::Tab))
        .expect("Tab should pass through");
}

// ============================================================================
// Edge Cases
// ============================================================================

/// Test CapsLock → Escape remapping (common use case).
///
/// This is a very common remapping that many users want.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_capslock_to_escape -- --ignored"]
fn test_capslock_to_escape() {
    let config = E2EConfig::simple_remap(KeyCode::CapsLock, KeyCode::Escape);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    let captured = harness
        .inject_and_capture(
            &TestEvents::tap(KeyCode::CapsLock),
            Duration::from_millis(100),
        )
        .expect("Failed to inject and capture");

    harness
        .verify(&captured, &TestEvents::tap(KeyCode::Escape))
        .expect("CapsLock should become Escape");
}

/// Test empty configuration (all keys passthrough).
///
/// Verifies that with no mappings configured, all keys pass through.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_empty_config_passthrough -- --ignored"]
fn test_empty_config_passthrough() {
    // No mappings
    let config = E2EConfig::default();
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture");

    harness
        .verify(&captured, &TestEvents::tap(KeyCode::A))
        .expect("A should pass through with empty config");
}

/// Test rapid key taps.
///
/// Verifies that rapid key presses are all captured and remapped correctly.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_rapid_key_taps -- --ignored"]
fn test_rapid_key_taps() {
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject 5 rapid taps
    let input: Vec<KeyEvent> = (0..5).flat_map(|_| TestEvents::tap(KeyCode::A)).collect();

    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(300))
        .expect("Failed to inject and capture");

    // Expect 5 taps of B
    let expected: Vec<KeyEvent> = (0..5).flat_map(|_| TestEvents::tap(KeyCode::B)).collect();
    harness
        .verify(&captured, &expected)
        .expect("All 5 rapid taps should be correctly remapped");
}

// ============================================================================
// Modifier State Tests - Requirement 5.2
// ============================================================================

/// Test modifier activation produces no output.
///
/// When a key is configured as a modifier (state change only), pressing it
/// should not produce any output events.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_modifier_no_output -- --ignored"]
fn test_modifier_no_output() {
    // CapsLock activates modifier 0 (no output)
    let config = E2EConfig::modifier(KeyCode::CapsLock, 0);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press and release modifier key
    let input = TestEvents::tap(KeyCode::CapsLock);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(150))
        .expect("Failed to inject and capture");

    // Expect no output events - modifier only sets internal state
    assert!(
        captured.is_empty(),
        "Modifier key should produce no output events, but got: {:?}",
        captured
    );
}

/// Test modifier key hold and release.
///
/// Verifies that pressing a modifier sets internal state, and releasing it
/// clears the state, all without producing any output.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_modifier_hold_release_no_output -- --ignored"]
fn test_modifier_hold_release_no_output() {
    let config = E2EConfig::modifier(KeyCode::CapsLock, 0);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Just press (hold) the modifier
    let press_input = TestEvents::press(KeyCode::CapsLock);
    let captured_press = harness
        .inject_and_capture(&press_input, Duration::from_millis(100))
        .expect("Failed to inject press");

    assert!(
        captured_press.is_empty(),
        "Modifier press should produce no output, but got: {:?}",
        captured_press
    );

    // Now release the modifier
    let release_input = TestEvents::release(KeyCode::CapsLock);
    let captured_release = harness
        .inject_and_capture(&release_input, Duration::from_millis(100))
        .expect("Failed to inject release");

    assert!(
        captured_release.is_empty(),
        "Modifier release should produce no output, but got: {:?}",
        captured_release
    );
}

// ============================================================================
// Lock State Tests - Requirement 5.3
// ============================================================================

/// Test lock toggle on first press.
///
/// Lock keys toggle internal state on press. The first press activates the lock.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_lock_toggle_no_output -- --ignored"]
fn test_lock_toggle_no_output() {
    // ScrollLock toggles lock 0 (no output)
    let config = E2EConfig::lock(KeyCode::ScrollLock, 0);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press lock key (toggles on)
    let input = TestEvents::tap(KeyCode::ScrollLock);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(150))
        .expect("Failed to inject and capture");

    // Expect no output - lock only sets internal state
    assert!(
        captured.is_empty(),
        "Lock key should produce no output events, but got: {:?}",
        captured
    );
}

/// Test lock toggle on second press.
///
/// The second press of a lock key should toggle the lock off.
/// Neither press nor release should produce output.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_lock_double_toggle_no_output -- --ignored"]
fn test_lock_double_toggle_no_output() {
    let config = E2EConfig::lock(KeyCode::ScrollLock, 0);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // First tap toggles lock ON
    let first_tap = TestEvents::tap(KeyCode::ScrollLock);
    let captured_first = harness
        .inject_and_capture(&first_tap, Duration::from_millis(100))
        .expect("Failed to inject first tap");

    assert!(
        captured_first.is_empty(),
        "First lock tap should produce no output, but got: {:?}",
        captured_first
    );

    // Second tap toggles lock OFF
    let second_tap = TestEvents::tap(KeyCode::ScrollLock);
    let captured_second = harness
        .inject_and_capture(&second_tap, Duration::from_millis(100))
        .expect("Failed to inject second tap");

    assert!(
        captured_second.is_empty(),
        "Second lock tap should produce no output, but got: {:?}",
        captured_second
    );
}

/// Test lock release produces no output.
///
/// Unlike modifiers (which are momentary), locks only toggle on press.
/// Release should be ignored and produce no output.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_lock_release_ignored -- --ignored"]
fn test_lock_release_ignored() {
    let config = E2EConfig::lock(KeyCode::ScrollLock, 0);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press the lock key
    let press_input = TestEvents::press(KeyCode::ScrollLock);
    let captured_press = harness
        .inject_and_capture(&press_input, Duration::from_millis(100))
        .expect("Failed to inject press");

    assert!(
        captured_press.is_empty(),
        "Lock press should produce no output"
    );

    // Release should also produce no output (locks toggle on press only)
    let release_input = TestEvents::release(KeyCode::ScrollLock);
    let captured_release = harness
        .inject_and_capture(&release_input, Duration::from_millis(100))
        .expect("Failed to inject release");

    assert!(
        captured_release.is_empty(),
        "Lock release should produce no output (locks toggle on press only)"
    );
}

// ============================================================================
// Conditional Mapping Tests - Requirement 5.4
// ============================================================================

/// Test conditional mapping with modifier active.
///
/// When modifier is held, the conditional mapping should be applied.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_conditional_with_modifier_active -- --ignored"]
fn test_conditional_with_modifier_active() {
    // CapsLock activates modifier 0, H→Left when modifier 0 is active
    let config =
        E2EConfig::with_modifier_layer(KeyCode::CapsLock, 0, vec![(KeyCode::H, KeyCode::Left)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press and hold modifier (CapsLock)
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to inject modifier press");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Now press H while modifier is held - should produce Left
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::H), Duration::from_millis(100))
        .expect("Failed to inject and capture H");

    // Expect Left key (H is remapped when modifier 0 is active)
    let expected = TestEvents::tap(KeyCode::Left);
    harness
        .verify(&captured, &expected)
        .expect("H should become Left when modifier is active");

    // Release modifier
    harness
        .inject(&TestEvents::release(KeyCode::CapsLock))
        .expect("Failed to inject modifier release");
}

/// Test conditional mapping without modifier (passthrough).
///
/// When modifier is not active, the conditional mapping should not apply,
/// and the key should pass through unchanged.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_conditional_without_modifier_passthrough -- --ignored"]
fn test_conditional_without_modifier_passthrough() {
    // CapsLock activates modifier 0, H→Left when modifier 0 is active
    let config =
        E2EConfig::with_modifier_layer(KeyCode::CapsLock, 0, vec![(KeyCode::H, KeyCode::Left)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press H without modifier active - should pass through as H
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::H), Duration::from_millis(100))
        .expect("Failed to inject and capture H");

    // Expect H unchanged (conditional not active)
    let expected = TestEvents::tap(KeyCode::H);
    harness
        .verify(&captured, &expected)
        .expect("H should pass through unchanged when modifier is not active");
}

/// Test conditional mapping with lock active.
///
/// When lock is toggled on, the conditional mapping should be applied.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_conditional_with_lock_active -- --ignored"]
fn test_conditional_with_lock_active() {
    // ScrollLock toggles lock 0, 1→F1 when lock 0 is active
    let config =
        E2EConfig::with_lock_layer(KeyCode::ScrollLock, 0, vec![(KeyCode::Num1, KeyCode::F1)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Toggle lock ON
    harness
        .inject(&TestEvents::tap(KeyCode::ScrollLock))
        .expect("Failed to toggle lock on");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Press 1 while lock is on - should produce F1
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::Num1), Duration::from_millis(100))
        .expect("Failed to inject and capture Num1");

    // Expect F1 key (Num1 is remapped when lock 0 is active)
    let expected = TestEvents::tap(KeyCode::F1);
    harness
        .verify(&captured, &expected)
        .expect("Num1 should become F1 when lock is active");
}

/// Test conditional mapping with lock inactive (passthrough).
///
/// When lock is toggled off, the conditional mapping should not apply.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_conditional_without_lock_passthrough -- --ignored"]
fn test_conditional_without_lock_passthrough() {
    // ScrollLock toggles lock 0, 1→F1 when lock 0 is active
    let config =
        E2EConfig::with_lock_layer(KeyCode::ScrollLock, 0, vec![(KeyCode::Num1, KeyCode::F1)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press 1 without lock active - should pass through as 1
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::Num1), Duration::from_millis(100))
        .expect("Failed to inject and capture Num1");

    // Expect Num1 unchanged (lock not active)
    let expected = TestEvents::tap(KeyCode::Num1);
    harness
        .verify(&captured, &expected)
        .expect("Num1 should pass through unchanged when lock is not active");
}

/// Test conditional mapping after lock toggle off.
///
/// After toggling lock off, the conditional mapping should no longer apply.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_conditional_after_lock_toggle_off -- --ignored"]
fn test_conditional_after_lock_toggle_off() {
    let config =
        E2EConfig::with_lock_layer(KeyCode::ScrollLock, 0, vec![(KeyCode::Num1, KeyCode::F1)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Toggle lock ON
    harness
        .inject(&TestEvents::tap(KeyCode::ScrollLock))
        .expect("Failed to toggle lock on");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Verify Num1 → F1 while lock is on
    let captured_on = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::Num1), Duration::from_millis(100))
        .expect("Failed to capture with lock on");
    harness
        .verify(&captured_on, &TestEvents::tap(KeyCode::F1))
        .expect("Num1 should become F1 when lock is on");

    // Toggle lock OFF
    harness
        .inject(&TestEvents::tap(KeyCode::ScrollLock))
        .expect("Failed to toggle lock off");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Verify Num1 passes through now (lock is off)
    let captured_off = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::Num1), Duration::from_millis(100))
        .expect("Failed to capture with lock off");
    harness
        .verify(&captured_off, &TestEvents::tap(KeyCode::Num1))
        .expect("Num1 should pass through when lock is off");
}

/// Test conditional mapping after modifier released.
///
/// After releasing modifier, the conditional mapping should no longer apply.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_conditional_after_modifier_released -- --ignored"]
fn test_conditional_after_modifier_released() {
    let config =
        E2EConfig::with_modifier_layer(KeyCode::CapsLock, 0, vec![(KeyCode::H, KeyCode::Left)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press modifier
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to press modifier");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Verify H → Left while modifier is held
    let captured_held = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::H), Duration::from_millis(100))
        .expect("Failed to capture with modifier held");
    harness
        .verify(&captured_held, &TestEvents::tap(KeyCode::Left))
        .expect("H should become Left when modifier is held");

    // Release modifier
    harness
        .inject(&TestEvents::release(KeyCode::CapsLock))
        .expect("Failed to release modifier");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Verify H passes through now (modifier released)
    let captured_released = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::H), Duration::from_millis(100))
        .expect("Failed to capture with modifier released");
    harness
        .verify(&captured_released, &TestEvents::tap(KeyCode::H))
        .expect("H should pass through when modifier is released");
}

/// Test multiple conditional mappings in same layer.
///
/// Verifies that multiple keys can be remapped within the same modifier layer.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_multiple_conditionals_same_layer -- --ignored"]
fn test_multiple_conditionals_same_layer() {
    // Vim-style navigation: CapsLock + HJKL → arrows
    let config = E2EConfig::with_modifier_layer(
        KeyCode::CapsLock,
        0,
        vec![
            (KeyCode::H, KeyCode::Left),
            (KeyCode::J, KeyCode::Down),
            (KeyCode::K, KeyCode::Up),
            (KeyCode::L, KeyCode::Right),
        ],
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press modifier
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to press modifier");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Test each navigation key
    let test_cases = [
        (KeyCode::H, KeyCode::Left),
        (KeyCode::J, KeyCode::Down),
        (KeyCode::K, KeyCode::Up),
        (KeyCode::L, KeyCode::Right),
    ];

    for (input_key, expected_key) in test_cases {
        let captured = harness
            .inject_and_capture(&TestEvents::tap(input_key), Duration::from_millis(100))
            .expect(&format!("Failed to capture {:?}", input_key));
        harness
            .verify(&captured, &TestEvents::tap(expected_key))
            .expect(&format!(
                "{:?} should become {:?} when modifier is held",
                input_key, expected_key
            ));
    }

    // Release modifier
    harness
        .inject(&TestEvents::release(KeyCode::CapsLock))
        .expect("Failed to release modifier");
}

// ============================================================================
// Modified Output Tests - Requirement 5.5
// ============================================================================

/// Test Shift+Key output sequence.
///
/// Verifies that a modified output mapping produces the correct event sequence:
/// Press: Press(LShift) → Press(key)
/// Release: Release(key) → Release(LShift)
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_modified_output_shift -- --ignored"]
fn test_modified_output_shift() {
    // A → Shift+1 (outputs '!' on most layouts)
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::Num1,
        true,  // shift
        false, // ctrl
        false, // alt
        false, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Test press event produces: Press(LShift), Press(Num1)
    let press_captured = harness
        .inject_and_capture(&TestEvents::press(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture press");

    let expected_press = vec![
        KeyEvent::Press(KeyCode::LShift),
        KeyEvent::Press(KeyCode::Num1),
    ];
    harness
        .verify(&press_captured, &expected_press)
        .expect("Press A should produce Press(LShift), Press(Num1)");

    // Test release event produces: Release(Num1), Release(LShift)
    let release_captured = harness
        .inject_and_capture(&TestEvents::release(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture release");

    let expected_release = vec![
        KeyEvent::Release(KeyCode::Num1),
        KeyEvent::Release(KeyCode::LShift),
    ];
    harness
        .verify(&release_captured, &expected_release)
        .expect("Release A should produce Release(Num1), Release(LShift)");
}

/// Test Ctrl+Key combination.
///
/// Verifies that Ctrl modifier is correctly applied to the output.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_modified_output_ctrl -- --ignored"]
fn test_modified_output_ctrl() {
    // A → Ctrl+C (copy shortcut)
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::C,
        false, // shift
        true,  // ctrl
        false, // alt
        false, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Test press event produces: Press(LCtrl), Press(C)
    let press_captured = harness
        .inject_and_capture(&TestEvents::press(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture press");

    let expected_press = vec![KeyEvent::Press(KeyCode::LCtrl), KeyEvent::Press(KeyCode::C)];
    harness
        .verify(&press_captured, &expected_press)
        .expect("Press A should produce Press(LCtrl), Press(C)");

    // Test release event produces: Release(C), Release(LCtrl)
    let release_captured = harness
        .inject_and_capture(&TestEvents::release(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture release");

    let expected_release = vec![
        KeyEvent::Release(KeyCode::C),
        KeyEvent::Release(KeyCode::LCtrl),
    ];
    harness
        .verify(&release_captured, &expected_release)
        .expect("Release A should produce Release(C), Release(LCtrl)");
}

/// Test Alt+Key combination.
///
/// Verifies that Alt modifier is correctly applied to the output.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_modified_output_alt -- --ignored"]
fn test_modified_output_alt() {
    // A → Alt+Tab (window switcher)
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::Tab,
        false, // shift
        false, // ctrl
        true,  // alt
        false, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Test press event produces: Press(LAlt), Press(Tab)
    let press_captured = harness
        .inject_and_capture(&TestEvents::press(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture press");

    let expected_press = vec![
        KeyEvent::Press(KeyCode::LAlt),
        KeyEvent::Press(KeyCode::Tab),
    ];
    harness
        .verify(&press_captured, &expected_press)
        .expect("Press A should produce Press(LAlt), Press(Tab)");

    // Test release event produces: Release(Tab), Release(LAlt)
    let release_captured = harness
        .inject_and_capture(&TestEvents::release(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture release");

    let expected_release = vec![
        KeyEvent::Release(KeyCode::Tab),
        KeyEvent::Release(KeyCode::LAlt),
    ];
    harness
        .verify(&release_captured, &expected_release)
        .expect("Release A should produce Release(Tab), Release(LAlt)");
}

/// Test Ctrl+Shift+Key multiple modifier combination.
///
/// Verifies correct ordering when multiple modifiers are used:
/// Press order: LShift → LCtrl → key
/// Release order: key → LCtrl → LShift
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_modified_output_ctrl_shift -- --ignored"]
fn test_modified_output_ctrl_shift() {
    // A → Ctrl+Shift+S (save as shortcut)
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::S,
        true,  // shift
        true,  // ctrl
        false, // alt
        false, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Test press event produces: Press(LShift), Press(LCtrl), Press(S)
    // Note: Order is shift, ctrl, alt, win, key
    let press_captured = harness
        .inject_and_capture(&TestEvents::press(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture press");

    let expected_press = vec![
        KeyEvent::Press(KeyCode::LShift),
        KeyEvent::Press(KeyCode::LCtrl),
        KeyEvent::Press(KeyCode::S),
    ];
    harness
        .verify(&press_captured, &expected_press)
        .expect("Press A should produce Press(LShift), Press(LCtrl), Press(S)");

    // Test release event produces: Release(S), Release(LCtrl), Release(LShift)
    // Reverse order of modifiers
    let release_captured = harness
        .inject_and_capture(&TestEvents::release(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture release");

    let expected_release = vec![
        KeyEvent::Release(KeyCode::S),
        KeyEvent::Release(KeyCode::LCtrl),
        KeyEvent::Release(KeyCode::LShift),
    ];
    harness
        .verify(&release_captured, &expected_release)
        .expect("Release A should produce Release(S), Release(LCtrl), Release(LShift)");
}

/// Test Ctrl+Alt+Key combination (common for system shortcuts).
///
/// Verifies correct ordering for Ctrl+Alt combinations.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_modified_output_ctrl_alt -- --ignored"]
fn test_modified_output_ctrl_alt() {
    // A → Ctrl+Alt+Delete style shortcut
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::Delete,
        false, // shift
        true,  // ctrl
        true,  // alt
        false, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Test press event produces: Press(LCtrl), Press(LAlt), Press(Delete)
    let press_captured = harness
        .inject_and_capture(&TestEvents::press(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture press");

    let expected_press = vec![
        KeyEvent::Press(KeyCode::LCtrl),
        KeyEvent::Press(KeyCode::LAlt),
        KeyEvent::Press(KeyCode::Delete),
    ];
    harness
        .verify(&press_captured, &expected_press)
        .expect("Press A should produce Press(LCtrl), Press(LAlt), Press(Delete)");

    // Test release event produces: Release(Delete), Release(LAlt), Release(LCtrl)
    let release_captured = harness
        .inject_and_capture(&TestEvents::release(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture release");

    let expected_release = vec![
        KeyEvent::Release(KeyCode::Delete),
        KeyEvent::Release(KeyCode::LAlt),
        KeyEvent::Release(KeyCode::LCtrl),
    ];
    harness
        .verify(&release_captured, &expected_release)
        .expect("Release A should produce Release(Delete), Release(LAlt), Release(LCtrl)");
}

/// Test all modifiers (Shift+Ctrl+Alt+Win).
///
/// Verifies correct ordering when all four modifiers are used.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_modified_output_all_modifiers -- --ignored"]
fn test_modified_output_all_modifiers() {
    // A → Shift+Ctrl+Alt+Win+Z (hypothetical super shortcut)
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::Z,
        true, // shift
        true, // ctrl
        true, // alt
        true, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Test press event produces: Press(LShift), Press(LCtrl), Press(LAlt), Press(LMeta), Press(Z)
    let press_captured = harness
        .inject_and_capture(&TestEvents::press(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture press");

    let expected_press = vec![
        KeyEvent::Press(KeyCode::LShift),
        KeyEvent::Press(KeyCode::LCtrl),
        KeyEvent::Press(KeyCode::LAlt),
        KeyEvent::Press(KeyCode::LMeta),
        KeyEvent::Press(KeyCode::Z),
    ];
    harness
        .verify(&press_captured, &expected_press)
        .expect("Press A should produce all modifiers then Z");

    // Test release event produces: Release(Z), Release(LMeta), Release(LAlt), Release(LCtrl), Release(LShift)
    let release_captured = harness
        .inject_and_capture(&TestEvents::release(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture release");

    let expected_release = vec![
        KeyEvent::Release(KeyCode::Z),
        KeyEvent::Release(KeyCode::LMeta),
        KeyEvent::Release(KeyCode::LAlt),
        KeyEvent::Release(KeyCode::LCtrl),
        KeyEvent::Release(KeyCode::LShift),
    ];
    harness
        .verify(&release_captured, &expected_release)
        .expect("Release A should release Z then all modifiers in reverse");
}

/// Test complete modified output tap sequence.
///
/// Verifies that a full tap (press+release) produces the complete correct sequence.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_modified_output_complete_tap -- --ignored"]
fn test_modified_output_complete_tap() {
    // A → Shift+1 complete tap
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::Num1,
        true,  // shift
        false, // ctrl
        false, // alt
        false, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject a complete tap (press + release)
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::A), Duration::from_millis(150))
        .expect("Failed to inject and capture tap");

    // Expected full sequence:
    // Press(LShift), Press(Num1), Release(Num1), Release(LShift)
    let expected = vec![
        KeyEvent::Press(KeyCode::LShift),
        KeyEvent::Press(KeyCode::Num1),
        KeyEvent::Release(KeyCode::Num1),
        KeyEvent::Release(KeyCode::LShift),
    ];
    harness
        .verify(&captured, &expected)
        .expect("Tap A should produce complete Shift+1 sequence");
}

/// Test multiple modified output taps in sequence.
///
/// Verifies that multiple modified output mappings work correctly in sequence.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_modified_output_multiple_taps -- --ignored"]
fn test_modified_output_multiple_taps() {
    // A → Shift+1
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::Num1,
        true,  // shift
        false, // ctrl
        false, // alt
        false, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // First tap
    let captured1 = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::A), Duration::from_millis(150))
        .expect("Failed to inject first tap");

    let expected_tap = vec![
        KeyEvent::Press(KeyCode::LShift),
        KeyEvent::Press(KeyCode::Num1),
        KeyEvent::Release(KeyCode::Num1),
        KeyEvent::Release(KeyCode::LShift),
    ];
    harness
        .verify(&captured1, &expected_tap)
        .expect("First tap should produce complete Shift+1 sequence");

    // Second tap - verify no state leakage
    let captured2 = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::A), Duration::from_millis(150))
        .expect("Failed to inject second tap");

    harness
        .verify(&captured2, &expected_tap)
        .expect("Second tap should produce same complete Shift+1 sequence");

    // Third tap
    let captured3 = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::A), Duration::from_millis(150))
        .expect("Failed to inject third tap");

    harness
        .verify(&captured3, &expected_tap)
        .expect("Third tap should produce same complete Shift+1 sequence");
}

/// Test modified output with unmapped key interleaving.
///
/// Verifies that modified output mappings don't affect unmapped keys.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_modified_output_with_passthrough -- --ignored"]
fn test_modified_output_with_passthrough() {
    // A → Shift+1, but B is unmapped
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::Num1,
        true,  // shift
        false, // ctrl
        false, // alt
        false, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // First test unmapped key passes through
    let b_captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::B), Duration::from_millis(100))
        .expect("Failed to inject B");
    harness
        .verify(&b_captured, &TestEvents::tap(KeyCode::B))
        .expect("B should pass through unchanged");

    // Then test modified output still works
    let a_captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::A), Duration::from_millis(150))
        .expect("Failed to inject A");
    let expected = vec![
        KeyEvent::Press(KeyCode::LShift),
        KeyEvent::Press(KeyCode::Num1),
        KeyEvent::Release(KeyCode::Num1),
        KeyEvent::Release(KeyCode::LShift),
    ];
    harness
        .verify(&a_captured, &expected)
        .expect("A should produce Shift+1");

    // And unmapped key still passes through after
    let c_captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::C), Duration::from_millis(100))
        .expect("Failed to inject C");
    harness
        .verify(&c_captured, &TestEvents::tap(KeyCode::C))
        .expect("C should pass through unchanged");
}
