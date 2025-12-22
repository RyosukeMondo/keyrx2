# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Core Runtime System

- **Runtime Data Structures**
  - `DeviceState` - 255-bit modifier and lock state tracking using BitVec
    - Sub-microsecond state updates (1.4ns for set_modifier, 2.2ns for toggle_lock)
    - Condition evaluation for conditional mappings (AllActive, NotActive)
    - Efficient bit manipulation with boundary validation (0-254, rejects 255+)
  - `KeyLookup` - O(1) average-case key-to-mapping resolution using HashMap
    - Ordered mapping search (conditionals before unconditional fallback)
    - Support for all mapping types (Simple, Modifier, Lock, TapHold, ModifiedOutput, Conditional)
  - `KeyEvent` - Type-safe keyboard event representation (Press/Release)
    - Derives Copy for efficient pass-by-value
    - Keycode extraction helper for pattern matching

- **Event Processing Engine**
  - `process_event()` - Core event processing function
    - Simple remapping (A→B) in ~17ns
    - Modifier activation/deactivation (no output) in ~10ns
    - Lock toggling with persistent state in ~2.2ns
    - ModifiedOutput sequences (Shift+Key, Ctrl+Key, etc.)
    - Conditional mapping evaluation with state-based resolution
    - Passthrough for unmapped keys in ~15ns
  - Deterministic event processing (same input + state → same output)
  - No panics on invalid input (error logging instead)

- **Platform Abstraction Layer**
  - `InputDevice` trait - Platform-agnostic input event capture
    - `next_event()` - Retrieve next keyboard event
    - `grab()` / `release()` - Exclusive device access control
  - `OutputDevice` trait - Platform-agnostic output event injection
    - `inject_event()` - Send keyboard events to OS
  - `MockInput` / `MockOutput` - Zero-dependency test implementations
    - FIFO event queue simulation
    - Grab/release flag tracking
    - Event capture for test verification
  - `DeviceError` enum - Comprehensive error handling
    - NotFound, PermissionDenied, EndOfStream, InjectionFailed, Io variants

- **Event Processor Orchestrator**
  - `EventProcessor<I, O>` - Generic event loop coordinator
    - `new()` - Initialize with config, input, and output devices
    - `process_one()` - Process single event (input → lookup → output)
    - `run()` - Main event loop until EndOfStream
  - Dependency injection for testability (generic over InputDevice/OutputDevice)
  - Structured JSON logging with timestamps and latency tracking
    - config_loaded, key_processed, state_transition, platform_error events
    - DEBUG level for per-event, INFO for lifecycle, ERROR for failures
  - Per-event latency measurement with std::time::Instant

- **Configuration Loader**
  - `load_config()` - Load and validate .krx binary files
  - `ConfigError` enum - Io and Deserialize error variants
  - Integration with keyrx_compiler serialization format

- **Testing Infrastructure**
  - 56 unit tests for runtime components (100% critical path coverage)
  - 15 integration tests for end-to-end workflows
  - Property-based tests with proptest:
    - Modifier state validity (rejects 255+, accepts 0-254)
    - Lock toggle cycles (OFF→ON→OFF)
    - Event processing determinism
    - No event loss during processing
  - Realistic scenario tests:
    - Vim navigation layer (CapsLock + HJKL → Arrow keys)
    - Lock persistence across key sequences
    - Multi-device independent state management
    - Complex multi-layer configurations

- **Performance Benchmarks**
  - Criterion-based benchmarks for critical paths:
    - key_lookup: ~4.7ns (O(1) HashMap access)
    - state_update_set_modifier: ~1.4ns
    - state_update_toggle_lock: ~2.2ns
    - process_event_simple: ~17ns (complete key remapping)
    - process_event_modifier: ~10ns (state update only)
    - process_event_passthrough: ~15ns (unmapped key)
  - All benchmarks meet <10μs latency requirement (actual: <20ns)

- **Fuzzing Infrastructure** (Optional)
  - cargo-fuzz integration for runtime components
  - Random event and config generation
  - Verified: no panics, no infinite loops, no crashes
  - Documented fuzzing setup and results

#### Compiler DSL Completion

- **Comprehensive DSL Validator Functions**
  - `parse_physical_key()` - Convert key names to KeyCode enum with VK_ prefix support
  - `parse_virtual_key()` - Parse virtual key names with VK_ prefix requirement
  - `parse_modifier_id()` - Validate custom modifier IDs (MD_00 through MD_FE)
  - `parse_lock_id()` - Validate custom lock IDs (LK_00 through LK_FE)
  - `parse_condition_string()` - Parse condition strings for when() and when_not()
  - Fuzzy matching for key names with "did you mean?" suggestions
  - Physical modifier name detection and rejection in custom modifiers

- **Complete DSL Function Set**
  - `map(from, to)` - Core mapping function supporting VK_/MD_/LK_ outputs
  - `tap_hold(key, tap, hold, threshold_ms)` - Dual-function keys
  - `with_shift(key)`, `with_ctrl(key)`, `with_alt(key)`, `with_win(key)` - Modifier helpers
  - `with_mods(key, shift, ctrl, alt, win)` - Multiple modifier helper
  - `when(condition, closure)` - Conditional mappings (single condition)
  - `when(conditions, closure)` - Conditional mappings (multiple conditions with AllActive)
  - `when_not(condition, closure)` - Negated conditional mappings
  - `device_start(pattern)` / `device_end()` - Device-specific configuration blocks

- **Enhanced CLI Subcommands**
  - `compile <input> [--output <output>]` - Compile Rhai scripts to .krx binaries
  - `verify <file>` - Validate .krx file integrity with detailed step-by-step output
  - `hash <file> [--verify]` - Extract and optionally verify embedded SHA256 hash
  - `parse <input> [--json]` - Parse and display configuration (human-readable or JSON)
  - Comprehensive error handling for all subcommands
  - Colored terminal output with NO_COLOR environment variable support

- **Error Formatting System**
  - Colored terminal output with syntax-highlighted error messages
  - Code snippet context (3 lines around error)
  - Caret (^) pointing to exact error column
  - Actionable help text with suggestions for each error type
  - Import chain display for errors in imported files
  - Specialized formatters for all error types:
    - Invalid prefix errors (detects MD_/VK_/LK_ issues)
    - Range errors (shows valid ID ranges)
    - Physical modifier errors (explains why physical names not allowed)
    - Missing prefix errors (suggests correct syntax)
    - Import not found errors (shows searched paths)
    - Circular import errors (displays import chain)
    - Resource limit errors (suggests simplification)

- **Import System**
  - Import resolution with relative and absolute paths
  - Circular import detection with detailed error messages
  - Multi-level recursive import support
  - Diamond dependency detection
  - Subdirectory import support
  - Import chain tracking for error reporting

- **Serialization System**
  - Binary .krx format with rkyv serialization
  - Magic bytes validation (KRXC)
  - Version field for forward compatibility
  - Embedded SHA256 hash for integrity verification
  - Deterministic serialization (same input → same output)
  - Round-trip serialization support

- **Documentation**
  - Comprehensive DSL Manual (docs/DSL_MANUAL.md)
    - Rhai syntax basics
    - Complete function reference with examples
    - Key naming reference (all VK_ codes, MD_00-MD_FE, LK_00-LK_FE)
    - Common patterns and best practices
    - Troubleshooting section with common errors
  - Six example configurations demonstrating:
    - 01-simple-remap.rhai - Basic key remapping
    - 02-capslock-escape.rhai - Classic CapsLock→Escape
    - 03-vim-navigation.rhai - Vim-style HJKL navigation layer
    - 04-dual-function-keys.rhai - Tap-hold configurations
    - 05-multiple-devices.rhai - Device-specific configurations
    - 06-advanced-layers.rhai - Complex multi-layer setup
  - Root README.md with quickstart guide
  - CI check for documentation accuracy (compiles all examples)

- **Testing Infrastructure**
  - 236 comprehensive tests across all modules
  - Property-based testing with proptest (700+ iterations)
  - End-to-end workflow tests (compile → verify → parse)
  - CLI integration tests with assert_cmd
  - Error formatting tests with ANSI code handling
  - Validator tests (100% coverage of validation logic)
  - DSL function tests (88 tests covering all functions)
  - Import resolution tests (circular detection, multilevel imports)
  - Serialization round-trip tests
  - 80.79% code coverage overall

### Changed

- Enhanced parser error messages with detailed context and suggestions
- Improved compilation error output with file:line:column locations
- Updated CLI to use clap derive macros for cleaner argument parsing

### Fixed

- Proper validation of modifier and lock ID ranges (00-FE, rejecting FF)
- Physical modifier names (LShift, RCtrl, etc.) correctly rejected in custom modifiers
- Deterministic compilation ensures consistent .krx output
- Circular import detection prevents infinite loops

## [0.1.0] - 2024-XX-XX

### Added

- Initial project structure with 4-crate workspace
- keyrx_core: Platform-agnostic remapping logic
- keyrx_compiler: Rhai-to-binary compiler
- keyrx_daemon: OS-level keyboard interception
- keyrx_ui: React-based web interface
- Basic Rhai DSL support
- MPHF-based O(1) key lookup
- DFA state machine for remapping
- 255-bit modifier/lock state tracking
- Web server with REST API and WebSocket support

[Unreleased]: https://github.com/yourusername/keyrx2/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/keyrx2/releases/tag/v0.1.0
