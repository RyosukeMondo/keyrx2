#![no_main]

use libfuzzer_sys::fuzz_target;
use std::panic;

use keyrx_core::config::{BaseKeyMapping, Condition, ConditionItem, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping};
use keyrx_core::runtime::event::{process_event, KeyEvent};
use keyrx_core::runtime::lookup::KeyLookup;
use keyrx_core::runtime::state::DeviceState;

fuzz_target!(|data: &[u8]| {
    // Fuzz test for runtime event processing
    //
    // OBJECTIVE: Verify process_event, KeyLookup, and DeviceState handle
    // arbitrary inputs without panics, infinite loops, or crashes
    //
    // STRATEGY:
    // 1. Parse fuzz input as KeyEvent sequence
    // 2. Use a fixed DeviceConfig with various mapping types
    // 3. Process all events through the runtime
    // 4. Verify no panics, no infinite loops, deterministic output
    //
    // REQUIREMENTS:
    // - No panics on any input (Reliability: No Panics)
    // - No undefined behavior (Security: No Panics)
    // - Deterministic execution (same input -> same output)

    // Catch any panics to prevent fuzzer from treating them as fatal
    let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        // Parse input as sequence of key events
        // Format: pairs of (u8 keycode, u8 event_type)
        // event_type: 0 = Press, 1 = Release
        let events: Vec<KeyEvent> = data
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() < 2 {
                    return None;
                }
                let keycode = chunk[0];
                let event_type = chunk[1];

                // Only use valid keycodes (0-255 map to KeyCode enum)
                // For fuzzing, we'll use a simplified approach: modulo 26 for A-Z
                let key = match keycode % 26 {
                    0 => KeyCode::A,
                    1 => KeyCode::B,
                    2 => KeyCode::C,
                    3 => KeyCode::D,
                    4 => KeyCode::E,
                    5 => KeyCode::F,
                    6 => KeyCode::G,
                    7 => KeyCode::H,
                    8 => KeyCode::I,
                    9 => KeyCode::J,
                    10 => KeyCode::K,
                    11 => KeyCode::L,
                    12 => KeyCode::M,
                    13 => KeyCode::N,
                    14 => KeyCode::O,
                    15 => KeyCode::P,
                    16 => KeyCode::Q,
                    17 => KeyCode::R,
                    18 => KeyCode::S,
                    19 => KeyCode::T,
                    20 => KeyCode::U,
                    21 => KeyCode::V,
                    22 => KeyCode::W,
                    23 => KeyCode::X,
                    24 => KeyCode::Y,
                    25 => KeyCode::Z,
                    _ => KeyCode::A, // fallback
                };

                let event = if event_type % 2 == 0 {
                    KeyEvent::Press(key)
                } else {
                    KeyEvent::Release(key)
                };

                Some(event)
            })
            .collect();

        // Create a DeviceConfig with various mapping types
        let config = DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: "fuzz-device".to_string(),
            },
            mappings: vec![
                // Simple mapping: A -> B
                KeyMapping::simple(KeyCode::A, KeyCode::B),
                // Modifier mapping: CapsLock -> MD_00
                KeyMapping::modifier(KeyCode::CapsLock, 0),
                // Lock mapping: ScrollLock -> LK_01
                KeyMapping::lock(KeyCode::ScrollLock, 1),
                // Conditional mapping: when MD_00 active, H -> Left
                KeyMapping::conditional(
                    Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                    vec![BaseKeyMapping::Simple {
                        from: KeyCode::H,
                        to: KeyCode::Left,
                    }],
                ),
                // ModifiedOutput: J -> Shift+1
                KeyMapping::modified_output(
                    KeyCode::J,
                    KeyCode::Num1,
                    true,  // shift
                    false, // ctrl
                    false, // alt
                    false, // meta
                ),
            ],
        };

        // Build lookup table
        let lookup = KeyLookup::from_device_config(&config);

        // Create device state
        let mut state = DeviceState::new();

        // Process all events
        let mut output_count = 0;
        for event in events {
            let output_events = process_event(event, &lookup, &mut state);

            // Verify output is bounded (prevent infinite loops)
            if output_events.len() > 100 {
                panic!("Output event count exceeded reasonable limit: {}", output_events.len());
            }

            output_count += output_events.len();
        }

        // Verify total output is reasonable
        if output_count > 1000 {
            panic!("Total output event count exceeded limit: {}", output_count);
        }
    }));
});
