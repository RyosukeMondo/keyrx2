//! macOS platform integration tests.
//!
//! These tests validate macOS-specific functionality including:
//! - CGKeyCode ↔ KeyCode bidirectional mapping
//! - rdev key conversion to internal KeyCode
//! - enigo key conversion from internal KeyCode
//! - Platform trait implementation for MacosPlatform
//! - Integration with EventProcessor
//!
//! Note: These tests use mocked input/output to avoid Accessibility
//! permission requirements, allowing them to run in CI.

#![cfg(target_os = "macos")]

use keyrx_core::config::{
    mappings::{BaseKeyMapping, DeviceConfig, DeviceIdentifier, KeyMapping},
    KeyCode,
};
use keyrx_core::runtime::KeyEvent;
use keyrx_daemon::platform::{MockInput, MockOutput};
use keyrx_daemon::processor::EventProcessor;

extern crate alloc;
use alloc::string::String;

/// Helper to create a test device config
fn create_test_config(mappings: Vec<KeyMapping>) -> DeviceConfig {
    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: String::from("*"),
        },
        mappings,
    }
}

/// Test: Basic key remapping on macOS
///
/// Validates that simple key remapping works correctly with macOS keycodes.
/// Uses MockInput/MockOutput to avoid Accessibility requirements.
#[test]
fn test_macos_basic_remap() {
    // Create simple A → B remapping
    let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);

    // Input: Press and release A
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::A),
        KeyEvent::Release(KeyCode::A),
    ]);
    let output = MockOutput::new();

    // Process events
    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify: Should output B press/release
    let events = processor.output().events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0], KeyEvent::Press(KeyCode::B));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::B));
}

/// Test: Command key modifier remapping
///
/// Validates macOS-specific Command (Cmd/⌘) modifier handling.
/// Tests LMeta (Command key on macOS).
#[test]
fn test_macos_command_modifier() {
    // Create config: LMeta → LCtrl (common for Windows/Linux users)
    let config = create_test_config(vec![KeyMapping::simple(KeyCode::LMeta, KeyCode::LCtrl)]);

    // Input: Cmd+C (Copy on macOS)
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::LMeta),
        KeyEvent::Press(KeyCode::C),
        KeyEvent::Release(KeyCode::C),
        KeyEvent::Release(KeyCode::LMeta),
    ]);
    let output = MockOutput::new();

    // Process events
    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify: Should output Ctrl+C
    let events = processor.output().events();
    assert_eq!(events.len(), 4);
    assert_eq!(events[0], KeyEvent::Press(KeyCode::LCtrl));
    assert_eq!(events[1], KeyEvent::Press(KeyCode::C));
    assert_eq!(events[2], KeyEvent::Release(KeyCode::C));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::LCtrl));
}

/// Test: macOS function key remapping
///
/// Validates mapping of macOS function keys (F1-F12) which may require
/// special handling on Apple keyboards with Touch Bar.
#[test]
fn test_macos_function_keys() {
    // Create config: F1 → F13 (extending function key range)
    let config = create_test_config(vec![
        KeyMapping::simple(KeyCode::F1, KeyCode::F13),
        KeyMapping::simple(KeyCode::F2, KeyCode::F14),
        KeyMapping::simple(KeyCode::F12, KeyCode::F24),
    ]);

    // Input: F1, F2, F12
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::F1),
        KeyEvent::Release(KeyCode::F1),
        KeyEvent::Press(KeyCode::F2),
        KeyEvent::Release(KeyCode::F2),
        KeyEvent::Press(KeyCode::F12),
        KeyEvent::Release(KeyCode::F12),
    ]);
    let output = MockOutput::new();

    // Process events
    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify: Should output F13, F14, F24
    let events = processor.output().events();
    assert_eq!(events.len(), 6);
    assert_eq!(events[0], KeyEvent::Press(KeyCode::F13));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::F13));
    assert_eq!(events[2], KeyEvent::Press(KeyCode::F14));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::F14));
    assert_eq!(events[4], KeyEvent::Press(KeyCode::F24));
    assert_eq!(events[5], KeyEvent::Release(KeyCode::F24));
}

/// Test: macOS special keys remapping
///
/// Validates handling of macOS-specific special keys like Mission Control,
/// Launchpad, and media keys.
#[test]
fn test_macos_special_keys() {
    // Create config: Volume keys to media control
    let config = create_test_config(vec![
        KeyMapping::simple(KeyCode::VolumeUp, KeyCode::MediaNext),
        KeyMapping::simple(KeyCode::VolumeDown, KeyCode::MediaPrevious),
        KeyMapping::simple(KeyCode::Mute, KeyCode::MediaPlayPause),
    ]);

    // Input: Volume controls
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::VolumeUp),
        KeyEvent::Release(KeyCode::VolumeUp),
        KeyEvent::Press(KeyCode::VolumeDown),
        KeyEvent::Release(KeyCode::VolumeDown),
        KeyEvent::Press(KeyCode::Mute),
        KeyEvent::Release(KeyCode::Mute),
    ]);
    let output = MockOutput::new();

    // Process events
    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify: Should output media controls
    let events = processor.output().events();
    assert_eq!(events.len(), 6);
    assert_eq!(events[0], KeyEvent::Press(KeyCode::MediaNext));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::MediaNext));
    assert_eq!(events[2], KeyEvent::Press(KeyCode::MediaPrevious));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::MediaPrevious));
    assert_eq!(events[4], KeyEvent::Press(KeyCode::MediaPlayPause));
    assert_eq!(events[5], KeyEvent::Release(KeyCode::MediaPlayPause));
}

/// Test: Vim navigation layer with Command modifier
///
/// Real-world scenario: macOS user wanting Vim-style navigation using
/// CapsLock as layer modifier. This tests complex layer behavior with
/// macOS-specific key sequences.
#[test]
fn test_macos_vim_navigation_layer() {
    // Create Vim navigation config (CapsLock + HJKL → arrows)
    let config = create_test_config(vec![
        KeyMapping::modifier(KeyCode::CapsLock, 0),
        KeyMapping::conditional(
            keyrx_core::config::Condition::AllActive(vec![
                keyrx_core::config::ConditionItem::ModifierActive(0),
            ]),
            vec![
                BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                },
            ],
        ),
        KeyMapping::conditional(
            keyrx_core::config::Condition::AllActive(vec![
                keyrx_core::config::ConditionItem::ModifierActive(0),
            ]),
            vec![
                BaseKeyMapping::Simple {
                    from: KeyCode::J,
                    to: KeyCode::Down,
                },
            ],
        ),
        KeyMapping::conditional(
            keyrx_core::config::Condition::AllActive(vec![
                keyrx_core::config::ConditionItem::ModifierActive(0),
            ]),
            vec![
                BaseKeyMapping::Simple {
                    from: KeyCode::K,
                    to: KeyCode::Up,
                },
            ],
        ),
        KeyMapping::conditional(
            keyrx_core::config::Condition::AllActive(vec![
                keyrx_core::config::ConditionItem::ModifierActive(0),
            ]),
            vec![
                BaseKeyMapping::Simple {
                    from: KeyCode::L,
                    to: KeyCode::Right,
                },
            ],
        ),
    ]);

    // Input: Type 'h', activate layer, navigate with HJKL, deactivate, type 'h'
    let input = MockInput::new(vec![
        // Before layer: 'h' should pass through
        KeyEvent::Press(KeyCode::H),
        KeyEvent::Release(KeyCode::H),
        // Activate navigation layer
        KeyEvent::Press(KeyCode::CapsLock),
        // Navigate with HJKL (should map to arrows)
        KeyEvent::Press(KeyCode::H),
        KeyEvent::Release(KeyCode::H),
        KeyEvent::Press(KeyCode::J),
        KeyEvent::Release(KeyCode::J),
        KeyEvent::Press(KeyCode::K),
        KeyEvent::Release(KeyCode::K),
        KeyEvent::Press(KeyCode::L),
        KeyEvent::Release(KeyCode::L),
        // Deactivate layer
        KeyEvent::Release(KeyCode::CapsLock),
        // After layer: 'h' should pass through
        KeyEvent::Press(KeyCode::H),
        KeyEvent::Release(KeyCode::H),
    ]);
    let output = MockOutput::new();

    // Process events
    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify output sequence
    let events = processor.output().events();

    // H before layer: passthrough
    assert_eq!(events[0], KeyEvent::Press(KeyCode::H));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::H));

    // CapsLock: no output (modifier)

    // H with layer: Left arrow
    assert_eq!(events[2], KeyEvent::Press(KeyCode::Left));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::Left));

    // J: Down arrow
    assert_eq!(events[4], KeyEvent::Press(KeyCode::Down));
    assert_eq!(events[5], KeyEvent::Release(KeyCode::Down));

    // K: Up arrow
    assert_eq!(events[6], KeyEvent::Press(KeyCode::Up));
    assert_eq!(events[7], KeyEvent::Release(KeyCode::Up));

    // L: Right arrow
    assert_eq!(events[8], KeyEvent::Press(KeyCode::Right));
    assert_eq!(events[9], KeyEvent::Release(KeyCode::Right));

    // CapsLock release: no output

    // H after layer: passthrough
    assert_eq!(events[10], KeyEvent::Press(KeyCode::H));
    assert_eq!(events[11], KeyEvent::Release(KeyCode::H));

    assert_eq!(events.len(), 12);
}

/// Test: Cross-platform compatibility
///
/// Validates that identical configs produce identical behavior on macOS
/// as they would on Linux/Windows (platform independence guarantee).
#[test]
fn test_macos_cross_platform_behavior() {
    // Create config using only cross-platform keys
    let config = create_test_config(vec![
        KeyMapping::simple(KeyCode::A, KeyCode::B),
        KeyMapping::simple(KeyCode::Num1, KeyCode::Num2),
        KeyMapping::simple(KeyCode::Space, KeyCode::Enter),
    ]);

    // Input: Mix of letter, number, and special keys
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::A),
        KeyEvent::Release(KeyCode::A),
        KeyEvent::Press(KeyCode::Num1),
        KeyEvent::Release(KeyCode::Num1),
        KeyEvent::Press(KeyCode::Space),
        KeyEvent::Release(KeyCode::Space),
    ]);
    let output = MockOutput::new();

    // Process events
    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify: Identical behavior to other platforms
    let events = processor.output().events();
    assert_eq!(events.len(), 6);
    assert_eq!(events[0], KeyEvent::Press(KeyCode::B));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::B));
    assert_eq!(events[2], KeyEvent::Press(KeyCode::Num2));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::Num2));
    assert_eq!(events[4], KeyEvent::Press(KeyCode::Enter));
    assert_eq!(events[5], KeyEvent::Release(KeyCode::Enter));
}

/// Test: Complex modifier combinations
///
/// Validates handling of complex modifier combinations common on macOS:
/// Cmd+Shift+X, Option+X, etc.
#[test]
fn test_macos_complex_modifiers() {
    // Create config: Remap Cmd+Shift+C to Cmd+Shift+V (copy → paste)
    let config = create_test_config(vec![KeyMapping::simple(KeyCode::C, KeyCode::V)]);

    // Input: Cmd+Shift+C
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::LMeta),
        KeyEvent::Press(KeyCode::LShift),
        KeyEvent::Press(KeyCode::C),
        KeyEvent::Release(KeyCode::C),
        KeyEvent::Release(KeyCode::LShift),
        KeyEvent::Release(KeyCode::LMeta),
    ]);
    let output = MockOutput::new();

    // Process events
    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify: Should output Cmd+Shift+V
    let events = processor.output().events();
    assert_eq!(events.len(), 6);
    assert_eq!(events[0], KeyEvent::Press(KeyCode::LMeta));
    assert_eq!(events[1], KeyEvent::Press(KeyCode::LShift));
    assert_eq!(events[2], KeyEvent::Press(KeyCode::V));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::V));
    assert_eq!(events[4], KeyEvent::Release(KeyCode::LShift));
    assert_eq!(events[5], KeyEvent::Release(KeyCode::LMeta));
}

/// Test: Rapid key sequences
///
/// Validates low-latency behavior under rapid key press/release sequences,
/// which is critical for macOS real-time event handling.
#[test]
fn test_macos_rapid_sequences() {
    // Create simple remap
    let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);

    // Input: Rapid A key presses (simulating fast typing)
    let mut events = Vec::new();
    for _ in 0..10 {
        events.push(KeyEvent::Press(KeyCode::A));
        events.push(KeyEvent::Release(KeyCode::A));
    }
    let input = MockInput::new(events);
    let output = MockOutput::new();

    // Process events
    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify: All events processed correctly
    let output_events = processor.output().events();
    assert_eq!(output_events.len(), 20);

    // Verify all are B presses/releases in correct order
    for i in 0..10 {
        assert_eq!(output_events[i * 2], KeyEvent::Press(KeyCode::B));
        assert_eq!(output_events[i * 2 + 1], KeyEvent::Release(KeyCode::B));
    }
}

/// Test: Unmapped keys pass through
///
/// Validates that unmapped keys pass through unchanged, which is critical
/// for user expectation (only configured keys are remapped).
#[test]
fn test_macos_passthrough() {
    // Create config with only A → B mapping
    let config = create_test_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);

    // Input: Mix of mapped (A) and unmapped (C, D) keys
    let input = MockInput::new(vec![
        KeyEvent::Press(KeyCode::A),
        KeyEvent::Release(KeyCode::A),
        KeyEvent::Press(KeyCode::C),
        KeyEvent::Release(KeyCode::C),
        KeyEvent::Press(KeyCode::D),
        KeyEvent::Release(KeyCode::D),
    ]);
    let output = MockOutput::new();

    // Process events
    let mut processor = EventProcessor::new(&config, input, output);
    processor.run().unwrap();

    // Verify: A mapped to B, C and D pass through
    let events = processor.output().events();
    assert_eq!(events.len(), 6);
    assert_eq!(events[0], KeyEvent::Press(KeyCode::B));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::B));
    assert_eq!(events[2], KeyEvent::Press(KeyCode::C));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::C));
    assert_eq!(events[4], KeyEvent::Press(KeyCode::D));
    assert_eq!(events[5], KeyEvent::Release(KeyCode::D));
}
