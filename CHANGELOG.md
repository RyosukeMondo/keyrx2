# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
