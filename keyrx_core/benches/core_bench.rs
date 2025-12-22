//! Criterion benchmarks for keyrx_core runtime performance
//!
//! Performance targets:
//! - Key lookup: <100μs (requirement: O(1) average-case HashMap lookup)
//! - State update: <10μs (requirement: sub-microsecond bit vector updates)
//! - End-to-end event processing: <1ms (requirement: overall latency budget)

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use keyrx_core::config::{DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping};
use keyrx_core::runtime::{process_event, DeviceState, KeyEvent, KeyLookup};

/// Create a realistic DeviceConfig with 100 mappings for benchmarking
fn create_realistic_config() -> DeviceConfig {
    let mut mappings = Vec::with_capacity(100);

    // Add 90 simple mappings (A-Z, 0-9, F1-F12, etc.)
    let keys = [
        KeyCode::A,
        KeyCode::B,
        KeyCode::C,
        KeyCode::D,
        KeyCode::E,
        KeyCode::F,
        KeyCode::G,
        KeyCode::H,
        KeyCode::I,
        KeyCode::J,
        KeyCode::K,
        KeyCode::L,
        KeyCode::M,
        KeyCode::N,
        KeyCode::O,
        KeyCode::P,
        KeyCode::Q,
        KeyCode::R,
        KeyCode::S,
        KeyCode::T,
        KeyCode::U,
        KeyCode::V,
        KeyCode::W,
        KeyCode::X,
        KeyCode::Y,
        KeyCode::Z,
        KeyCode::Num0,
        KeyCode::Num1,
        KeyCode::Num2,
        KeyCode::Num3,
        KeyCode::Num4,
        KeyCode::Num5,
        KeyCode::Num6,
        KeyCode::Num7,
        KeyCode::Num8,
        KeyCode::Num9,
        KeyCode::F1,
        KeyCode::F2,
        KeyCode::F3,
        KeyCode::F4,
        KeyCode::F5,
        KeyCode::F6,
        KeyCode::F7,
        KeyCode::F8,
        KeyCode::F9,
        KeyCode::F10,
        KeyCode::F11,
        KeyCode::F12,
        KeyCode::Escape,
        KeyCode::Tab,
        KeyCode::Space,
        KeyCode::Enter,
        KeyCode::Backspace,
        KeyCode::Delete,
        KeyCode::Home,
        KeyCode::End,
        KeyCode::PageUp,
        KeyCode::PageDown,
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Insert,
        KeyCode::Minus,
        KeyCode::Equal,
        KeyCode::LeftBracket,
        KeyCode::RightBracket,
        KeyCode::Backslash,
        KeyCode::Semicolon,
        KeyCode::Quote,
        KeyCode::Comma,
        KeyCode::Period,
        KeyCode::Slash,
        KeyCode::Grave,
        KeyCode::PrintScreen,
        KeyCode::ScrollLock,
        KeyCode::Pause,
        KeyCode::NumLock,
        KeyCode::NumpadDivide,
        KeyCode::NumpadMultiply,
        KeyCode::NumpadSubtract,
        KeyCode::NumpadAdd,
        KeyCode::NumpadEnter,
        KeyCode::Numpad0,
        KeyCode::Numpad1,
        KeyCode::Numpad2,
        KeyCode::Numpad3,
        KeyCode::Numpad4,
        KeyCode::Numpad5,
        KeyCode::Numpad6,
        KeyCode::Numpad7,
    ];

    // Add simple mappings (first 90 keys)
    for (i, &key) in keys.iter().enumerate().take(90) {
        let target_key = keys[(i + 1) % keys.len()];
        mappings.push(KeyMapping::simple(key, target_key));
    }

    // Add 5 modifier mappings
    mappings.push(KeyMapping::modifier(KeyCode::CapsLock, 0));
    mappings.push(KeyMapping::modifier(KeyCode::LCtrl, 1));
    mappings.push(KeyMapping::modifier(KeyCode::RCtrl, 2));
    mappings.push(KeyMapping::modifier(KeyCode::LShift, 3));
    mappings.push(KeyMapping::modifier(KeyCode::RShift, 4));

    // Add 5 lock mappings
    mappings.push(KeyMapping::lock(KeyCode::ScrollLock, 0));
    mappings.push(KeyMapping::lock(KeyCode::NumLock, 1));
    mappings.push(KeyMapping::lock(KeyCode::Pause, 2));
    mappings.push(KeyMapping::lock(KeyCode::PrintScreen, 3));
    mappings.push(KeyMapping::lock(KeyCode::Insert, 4));

    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: "Benchmark Keyboard".to_string(),
        },
        mappings,
    }
}

/// Benchmark: Key lookup time (<100μs target)
///
/// Measures KeyLookup::find_mapping performance with realistic config (100 mappings)
fn benchmark_key_lookup(c: &mut Criterion) {
    let config = create_realistic_config();
    let lookup = KeyLookup::from_device_config(&config);
    let state = DeviceState::new();

    c.bench_function("key_lookup", |b| {
        b.iter(|| {
            // Test lookup for a key that exists in the table
            let result = lookup.find_mapping(black_box(KeyCode::H), black_box(&state));
            black_box(result);
        })
    });
}

/// Benchmark: State update time (<10μs target)
///
/// Measures DeviceState::set_modifier and toggle_lock performance
fn benchmark_state_update(c: &mut Criterion) {
    let mut state = DeviceState::new();

    c.bench_function("state_update_set_modifier", |b| {
        b.iter(|| {
            state.set_modifier(black_box(42));
        })
    });

    c.bench_function("state_update_toggle_lock", |b| {
        b.iter(|| {
            state.toggle_lock(black_box(42));
        })
    });
}

/// Benchmark: End-to-end event processing (<1ms target)
///
/// Measures complete process_event pipeline (lookup + state + processing)
fn benchmark_process_event(c: &mut Criterion) {
    let config = create_realistic_config();
    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    c.bench_function("process_event_simple", |b| {
        b.iter(|| {
            // Process a simple remapping (most common case)
            let event = KeyEvent::Press(black_box(KeyCode::H));
            let result = process_event(event, black_box(&lookup), black_box(&mut state));
            black_box(result);
        })
    });

    c.bench_function("process_event_modifier", |b| {
        b.iter(|| {
            // Process a modifier mapping (state update)
            let event = KeyEvent::Press(black_box(KeyCode::CapsLock));
            let result = process_event(event, black_box(&lookup), black_box(&mut state));
            black_box(result);
        })
    });

    c.bench_function("process_event_passthrough", |b| {
        b.iter(|| {
            // Process an unmapped key (passthrough case)
            let event = KeyEvent::Press(black_box(KeyCode::Numpad8));
            let result = process_event(event, black_box(&lookup), black_box(&mut state));
            black_box(result);
        })
    });
}

criterion_group!(
    benches,
    benchmark_key_lookup,
    benchmark_state_update,
    benchmark_process_event
);
criterion_main!(benches);
