# AI Agent Development Guide

## AI-Agent Quick Start

### 1. Verify Environment

**Check Rust toolchain:**
```bash
rustc --version  # Requires Rust 1.70+
cargo --version
```

**Check Node.js (for keyrx_ui):**
```bash
node --version   # Requires Node.js 18+
npm --version
```

### 2. Setup Development Environment

**Install required tools:**
```bash
make setup
```

This installs:
- `cargo-watch` - Continuous build on file changes
- `cargo-tarpaulin` - Code coverage analysis
- `cargo-fuzz` - Fuzzing support
- `wasm-pack` - WASM compilation
- Git pre-commit hooks - Automated quality gates

### 3. Run First Build

**Build all crates:**
```bash
make build
# Or: scripts/build.sh
```

Expected output:
```
[2025-12-21 10:30:45] [INFO] Building workspace...
[2025-12-21 10:30:50] === accomplished ===
```

### 4. Run Tests

**Execute all tests:**
```bash
make test
# Or: scripts/test.sh
```

### 5. Run Verification

**Run complete quality checks (clippy, fmt, tests, coverage):**
```bash
make verify
# Or: scripts/verify.sh
```

This runs:
- Cargo build (clean workspace build)
- Clippy linting (-D warnings, treats warnings as errors)
- Rustfmt check (code formatting)
- Cargo test (all unit and integration tests)
- Coverage analysis (80% minimum required)

**Success criteria:** All checks must pass before committing code.

## Project Structure

### 4-Crate Workspace

```
keyrx2/
├── Cargo.toml              # Workspace root
├── Makefile                # Top-level commands
├── .github/workflows/      # CI/CD automation
│   ├── ci.yml              # Automated verification
│   └── release.yml         # Release builds
├── keyrx_core/             # Core library (no_std, WASM-compatible)
│   ├── src/
│   │   ├── lib.rs          # Public API exports
│   │   ├── config.rs       # rkyv-serialized config structures
│   │   ├── lookup.rs       # MPHF-based O(1) key lookup
│   │   ├── dfa.rs          # Deterministic Finite Automaton
│   │   ├── state.rs        # 255-bit modifier/lock state
│   │   └── simulator.rs    # Deterministic Simulation Testing
│   ├── benches/            # Criterion benchmarks
│   └── fuzz/               # cargo-fuzz targets
├── keyrx_compiler/         # Rhai-to-binary compiler (CLI)
│   ├── src/
│   │   ├── main.rs         # CLI entry point
│   │   ├── parser.rs       # Rhai AST evaluation
│   │   ├── mphf_gen.rs     # MPHF generation
│   │   ├── dfa_gen.rs      # DFA compilation
│   │   └── serialize.rs    # rkyv binary output
│   └── tests/integration/  # Integration tests
├── keyrx_daemon/           # OS-specific daemon + web server
│   ├── src/
│   │   ├── main.rs         # Daemon entry point
│   │   ├── platform/       # Platform-specific code
│   │   │   ├── mod.rs      # Platform trait abstraction
│   │   │   ├── linux.rs    # evdev/uinput
│   │   │   └── windows.rs  # Low-Level Hooks
│   │   └── web/            # Embedded web server (optional)
│   │       ├── mod.rs      # axum server setup
│   │       ├── api.rs      # REST API
│   │       ├── ws.rs       # WebSocket handler
│   │       └── static_files.rs # Serve embedded UI
│   └── ui_dist/            # Embedded UI files
├── keyrx_ui/               # React + WASM frontend
│   ├── src/
│   │   ├── App.tsx         # Root component
│   │   ├── components/     # React components
│   │   ├── wasm/           # WASM bindings
│   │   └── hooks/          # React hooks
│   └── vite.config.ts      # Vite bundler config
└── scripts/                # Build/test/launch automation
    ├── build.sh            # Build workspace
    ├── verify.sh           # Quality checks
    ├── test.sh             # Run tests
    ├── launch.sh           # Start daemon
    ├── setup_hooks.sh      # Install pre-commit hooks
    ├── lib/common.sh       # Shared utilities
    ├── logs/               # Timestamped execution logs
    └── CLAUDE.md           # Script documentation
```

### Crate Purposes

| Crate | Type | Purpose | Key Dependencies |
|-------|------|---------|------------------|
| `keyrx_core` | Library (no_std) | Platform-agnostic remapping logic | rkyv, boomphf, fixedbitset, arrayvec |
| `keyrx_compiler` | Binary | Compile Rhai configs to .krx binaries | rhai, serde, clap |
| `keyrx_daemon` | Binary | OS-level keyboard interception + web server | evdev (Linux), windows-sys (Windows), axum (web) |
| `keyrx_ui` | Frontend | React-based web interface with WASM | React 18+, TypeScript 5+, Vite |

## Code Quality Rules

### File and Function Size Limits

**Enforced by clippy and pre-commit hooks:**

- **Maximum 500 lines per file** (excluding comments and blank lines)
  - If exceeded: Extract helper modules or split into sub-modules
- **Maximum 50 lines per function**
  - If exceeded: Extract helper functions, apply SLAP (Single Level of Abstraction Principle)

### Test Coverage

- **Minimum 80% coverage** (enforced by verify.sh)
- **Minimum 90% coverage for critical paths** (keyrx_core)
- Coverage measured by `cargo tarpaulin`
- Coverage reports uploaded to CI artifacts

### Code Quality Checks

All checks enforced by pre-commit hooks and CI:

1. **Clippy**: `cargo clippy --workspace -- -D warnings`
   - Treats warnings as errors
   - Enforces best practices and idiomatic code
2. **Rustfmt**: `cargo fmt --check`
   - Consistent code formatting
   - Fails if code is not formatted
3. **Tests**: `cargo test --workspace`
   - All tests must pass
   - No ignored tests in production code

## Architecture Patterns

### SOLID Principles

**S - Single Responsibility Principle:**
```rust
// Good: Each module has one clear purpose
// lookup.rs - handles MPHF lookup only
// dfa.rs - handles DFA state transitions only

// Bad: lookup.rs mixing lookup + DFA + state management
```

**O - Open/Closed Principle:**
```rust
// Good: Extensible via traits
pub trait Platform {
    fn capture_input(&mut self) -> Result<KeyEvent>;
    fn inject_output(&mut self, event: KeyEvent) -> Result<()>;
}

// Add new platforms by implementing trait, no changes to core
```

**L - Liskov Substitution Principle:**
```rust
// Platform implementations must be substitutable
fn process_events<P: Platform>(platform: &mut P) {
    // Works with any Platform implementation
}
```

**I - Interface Segregation Principle:**
```rust
// Good: Small, focused traits
pub trait EventCapture {
    fn capture(&mut self) -> Result<KeyEvent>;
}

pub trait EventInjection {
    fn inject(&mut self, event: KeyEvent) -> Result<()>;
}

// Bad: One giant trait with all methods
```

**D - Dependency Inversion Principle:**
```rust
// Good: Depend on abstractions (traits), not concrete types
pub fn process_events<S: EventStream>(stream: &mut S) {
    // Testable, mockable
}

// Bad: Hard-coded concrete dependency
pub fn process_events() {
    let stream = evdev::open(); // NOT testable
}
```

### Dependency Injection Pattern

**All external dependencies injected:**
```rust
// Platform-specific code abstracted via traits
pub trait Platform {
    fn capture_input(&mut self) -> Result<KeyEvent>;
    fn inject_output(&mut self, event: KeyEvent) -> Result<()>;
}

// Inject dependency for testability
pub struct Daemon<P: Platform> {
    platform: P,
}

impl<P: Platform> Daemon<P> {
    pub fn new(platform: P) -> Self {
        Self { platform }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let event = self.platform.capture_input()?;
            let action = self.process(event)?;
            self.platform.inject_output(action)?;
        }
    }
}

// Test with mock implementation
#[cfg(test)]
mod tests {
    struct MockPlatform;
    impl Platform for MockPlatform {
        fn capture_input(&mut self) -> Result<KeyEvent> { /* ... */ }
        fn inject_output(&mut self, event: KeyEvent) -> Result<()> { /* ... */ }
    }

    #[test]
    fn test_daemon() {
        let mut daemon = Daemon::new(MockPlatform);
        // Test without real OS dependencies
    }
}
```

### SSOT (Single Source of Truth)

**Configuration:**
- `.krx` binary file is the ONLY config source
- Daemon, UI, tests all read same `.krx` file
- No duplication in JSON, TOML, or other formats
- Hash-based verification ensures integrity

**State:**
- `ExtendedState` struct is the ONLY state representation
- No shadow copies, no stale caches
- All components reference the same state

**Example:**
```rust
// Good: Single source of truth
pub struct Config {
    krx_data: &'static [u8],  // Memory-mapped .krx file
}

// Bad: Multiple representations
// pub struct JsonConfig { /* ... */ }
// pub struct TomlConfig { /* ... */ }
```

### KISS (Keep It Simple, Stupid)

**Prefer simplicity over premature optimization:**
```rust
// Good: Simple, clear logic
pub fn is_modifier_active(state: &State, modifier_id: u8) -> bool {
    state.modifiers.contains(modifier_id)
}

// Bad: Over-engineered
pub fn is_modifier_active<S: StateProvider, M: ModifierId>(
    state: &S,
    modifier: M,
    cache: &mut ModifierCache,
) -> Result<bool, ModifierError> {
    // Unnecessary complexity
}
```

**Only add complexity when required:**
- Don't add features not explicitly needed
- Don't abstract until you have 3+ similar cases
- Don't optimize without profiling first

## Naming Conventions

### Rust Code

| Element | Convention | Example |
|---------|-----------|---------|
| Modules | `snake_case` | `mphf_gen`, `static_files` |
| Files | `snake_case.rs` | `lookup.rs`, `dfa_gen.rs` |
| Functions | `snake_case` | `load_config()`, `process_event()` |
| Structs | `PascalCase` | `ExtendedState`, `EventStream` |
| Enums | `PascalCase` | `EventType`, `ModifierState` |
| Traits | `PascalCase` | `Platform`, `EventCapture` |
| Constants | `UPPER_SNAKE_CASE` | `MAX_MODIFIERS`, `DEFAULT_PORT` |
| Variables | `snake_case` | `modifier_state`, `event_queue` |
| Type parameters | `PascalCase` or single uppercase | `T`, `EventType` |

### TypeScript/React Code

| Element | Convention | Example |
|---------|-----------|---------|
| Components | `PascalCase` | `KeyboardVisualizer`, `DFADiagram` |
| Files (components) | `PascalCase.tsx` | `App.tsx`, `KeyboardVisualizer.tsx` |
| Functions | `camelCase` | `connectToDaemon()`, `parseEvent()` |
| Hooks | `use[Feature]` | `useSimulator()`, `useDaemon()` |
| Files (hooks) | `use[Feature].ts` | `useSimulator.ts`, `useDaemon.ts` |
| Interfaces/Types | `PascalCase` | `DaemonState`, `KeyEvent` |
| Constants | `UPPER_SNAKE_CASE` | `WS_PORT`, `MAX_RETRIES` |
| Variables | `camelCase` | `eventQueue`, `modifierState` |

### Files and Directories

| Element | Convention | Example |
|---------|-----------|---------|
| Rust modules | `snake_case.rs` | `config.rs`, `lookup.rs` |
| Test files | `[module]_test.rs` | `dfa_test.rs` |
| Benchmark files | `[feature]_bench.rs` | `lookup_bench.rs` |
| Scripts | `lowercase.sh` | `build.sh`, `verify.sh` |
| Log files | `[script]_[epoch].log` | `build_1766292917.log` |

## Import Patterns

### Rust Import Order

```rust
// 1. Standard library
use std::collections::HashMap;
use std::fs::File;

// 2. External dependencies (alphabetically)
use rkyv::{Archive, Serialize};
use serde::Deserialize;

// 3. Internal workspace crates
use keyrx_core::{EventStream, State};
use keyrx_compiler::Parser;

// 4. Current crate modules (relative)
use crate::config::Config;
use super::utils;

// Example from keyrx_daemon/src/platform/linux.rs:
use std::os::unix::io::AsRawFd;

use evdev::{Device, InputEventKind};
use nix::ioctl_write_int_bad;

use keyrx_core::{EventStream, KeyEvent};

use crate::platform::Platform;
```

### TypeScript Import Order

```typescript
// 1. React and framework
import React, { useState, useEffect } from 'react';

// 2. External dependencies
import { WebSocket } from 'ws';

// 3. Internal modules (absolute from src/)
import { WasmCore } from '@/wasm/core';
import type { KeyEvent } from '@/types';

// 4. Relative imports
import { Button } from './Button';

// 5. Styles (last)
import './App.css';

// Example from keyrx_ui/src/App.tsx:
import React, { useState } from 'react';

import { WasmCore } from '@/wasm/core';
import { useDaemon } from '@/hooks/useDaemon';

import { KeyboardVisualizer } from './components/KeyboardVisualizer';

import './App.css';
```

### Module Organization

**Absolute imports between crates:**
```rust
// Good: Workspace-relative imports
use keyrx_core::config::Config;

// Bad: Relative paths between crates
use ../keyrx_core/src/config.rs;  // DON'T DO THIS
```

**Re-exports for public API:**
```rust
// keyrx_core/src/lib.rs
pub use self::config::Config;
pub use self::dfa::DFA;
pub use self::state::State;

// Users import from crate root:
use keyrx_core::{Config, DFA, State};
```

**Feature gates for optional dependencies:**
```rust
#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "linux")]
pub mod linux;
```

## Common Tasks

### How to Add a New Module

**Example: Adding a new module `validator.rs` to keyrx_core**

1. **Create the module file:**
```bash
touch keyrx_core/src/validator.rs
```

2. **Declare the module in lib.rs:**
```rust
// keyrx_core/src/lib.rs
pub mod validator;
```

3. **Implement the module:**
```rust
// keyrx_core/src/validator.rs
//! Input validation for configuration data.

use crate::config::Config;

/// Validates a configuration for correctness.
pub fn validate_config(config: &Config) -> Result<(), ValidationError> {
    // Validation logic
    Ok(())
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidModifier,
    InvalidKeyCode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_config() {
        // Test validation logic
    }
}
```

4. **Re-export if public API:**
```rust
// keyrx_core/src/lib.rs
pub use self::validator::{validate_config, ValidationError};
```

5. **Add tests:**
```bash
# Tests are in the same file (see above)
# Or create integration test:
touch keyrx_core/tests/validator_integration.rs
```

6. **Verify:**
```bash
make verify
```

### How to Add a Test

**Unit test (in same file as code):**
```rust
// keyrx_core/src/lookup.rs
pub fn lookup_key(hash: u64) -> Option<KeyCode> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_key_found() {
        let result = lookup_key(12345);
        assert!(result.is_some());
    }

    #[test]
    fn test_lookup_key_not_found() {
        let result = lookup_key(99999);
        assert!(result.is_none());
    }
}
```

**Integration test (separate file):**
```bash
# Create test file
touch keyrx_core/tests/dfa_integration.rs
```

```rust
// keyrx_core/tests/dfa_integration.rs
use keyrx_core::{DFA, State, KeyEvent};

#[test]
fn test_dfa_tap_hold_sequence() {
    let mut dfa = DFA::new();
    let mut state = State::new();

    // Test tap/hold logic
    let event1 = KeyEvent::new(/* ... */);
    let result = dfa.process(&mut state, event1);
    assert_eq!(result, /* expected */);
}
```

**Run the tests:**
```bash
# Run all tests
make test

# Run specific test
cargo test test_lookup_key_found

# Run tests in a specific crate
cargo test -p keyrx_core

# Run integration tests only
scripts/test.sh --integration
```

### How to Run Specific Tests

```bash
# Run all tests
cargo test

# Run tests matching a pattern
cargo test lookup

# Run tests in a specific crate
cargo test -p keyrx_core

# Run a specific test
cargo test test_lookup_key_found

# Run only unit tests (lib tests)
scripts/test.sh --unit

# Run only integration tests
scripts/test.sh --integration

# Run with verbose output
cargo test -- --nocapture

# Run benchmarks (requires nightly)
scripts/test.sh --bench
```

### How to Add a Dependency

**Add to Cargo.toml:**
```bash
cd keyrx_core
cargo add rkyv --features validation
```

Or manually edit `Cargo.toml`:
```toml
[dependencies]
rkyv = { version = "0.7", features = ["validation"] }
```

**For platform-specific dependencies:**
```toml
[target.'cfg(target_os = "linux")'.dependencies]
evdev = "0.12"

[target.'cfg(target_os = "windows")'.dependencies]
windows-sys = { version = "0.48", features = ["Win32_UI_Input_KeyboardAndMouse"] }
```

**Verify the dependency:**
```bash
make build
```

### How to Fix Clippy Warnings

**Run clippy to see warnings:**
```bash
cargo clippy --workspace -- -D warnings
```

**Common warnings and fixes:**

1. **Needless borrow:**
```rust
// Warning: needless borrow
foo(&x);

// Fix:
foo(x);
```

2. **Unnecessary mut:**
```rust
// Warning: variable does not need to be mutable
let mut x = 5;

// Fix:
let x = 5;
```

3. **Match can be simplified:**
```rust
// Warning: match expression can be simplified
match result {
    Ok(val) => Some(val),
    Err(_) => None,
}

// Fix:
result.ok()
```

4. **Unused variable:**
```rust
// Warning: unused variable
let unused = 5;

// Fix: Remove or prefix with underscore
let _unused = 5;
```

### How to Format Code

**Format all code:**
```bash
cargo fmt
```

**Check formatting without modifying:**
```bash
cargo fmt --check
```

**Format is automatically checked by:**
- Pre-commit hooks
- CI/CD pipeline
- `make verify`

## Troubleshooting

### Build Failures

**Error: "error: could not compile"**
- **Cause**: Syntax error in Rust code
- **Fix**: Read error message, locate file and line number, fix syntax
- **Example**: Missing semicolon, unclosed brace, type mismatch

**Error: "error[E0425]: cannot find value"**
- **Cause**: Undefined variable or function
- **Fix**: Check imports, add missing dependency, or define the item
- **Example**: Forgot to import `use std::collections::HashMap;`

**Error: "error: package collision"**
- **Cause**: Duplicate dependency versions
- **Fix**: Run `cargo update` or specify exact versions in Cargo.toml

### Test Failures

**Error: "test result: FAILED"**
- **Cause**: Test assertion failed
- **Fix**:
  1. Run with verbose output: `cargo test -- --nocapture`
  2. Read assertion message
  3. Fix implementation or test
  4. Re-run: `make test`

**Error: Coverage below 80%**
- **Cause**: Insufficient test coverage
- **Fix**:
  1. Run `cargo tarpaulin` to see coverage report
  2. Identify uncovered lines
  3. Add tests for uncovered code paths
  4. Re-run: `make verify`

### Pre-Commit Hook Blocks Commit

**Error: "Pre-commit checks failed. Commit aborted."**
- **Cause**: Code fails quality checks (clippy, fmt, tests)
- **Fix**:
  1. Run `make verify` to see detailed errors
  2. Fix reported issues
  3. Re-run `make verify` to confirm fixes
  4. Commit again

**Bypass hook (NOT recommended):**
```bash
git commit --no-verify
```

### CI/CD Failures

**Error: "Clippy check failed"**
- **Cause**: Code has warnings treated as errors
- **Fix**: Run `cargo clippy --workspace -- -D warnings` locally, fix warnings

**Error: "Format check failed"**
- **Cause**: Code is not formatted
- **Fix**: Run `cargo fmt`, commit formatted code

**Error: "Tests failed"**
- **Cause**: Tests fail on CI but pass locally
- **Fix**: Check for platform-specific issues, ensure tests are deterministic (no wall-clock time, no random data)

### Missing Tools

**Error: "Required tool 'cargo-watch' not found"**
- **Cause**: Development tool not installed
- **Fix**: Run `make setup` to install all required tools

**Manual installation:**
```bash
cargo install cargo-watch
cargo install cargo-tarpaulin
cargo install cargo-fuzz
cargo install wasm-pack
```

### Script Errors

**Error: "=== failed ==="**
- **Cause**: Script operation failed
- **Fix**:
  1. Check log file: `scripts/logs/[script]_[timestamp].log`
  2. Read error messages
  3. Fix underlying issue
  4. Re-run script

**Get detailed output:**
```bash
# Run without --quiet flag
scripts/build.sh

# View log file
cat scripts/logs/build_*.log | tail -50
```

## Advanced Usage

### Continuous Development Mode

**Watch mode for automatic rebuilds:**
```bash
scripts/build.sh --watch
```

Code changes trigger automatic rebuilds. Press Ctrl+C to stop.

### JSON Output for CI/CD

**Get machine-readable output:**
```bash
scripts/verify.sh --json
```

Parse output with `jq`:
```bash
scripts/verify.sh --json | jq '.checks.coverage'
```

### Custom Log Files

**Specify custom log location:**
```bash
scripts/build.sh --log-file /tmp/my-build.log
```

### Debug Mode

**Enable debug logging:**
```bash
scripts/launch.sh --debug
```

Outputs debug-level messages for troubleshooting.

### Headless Mode

**Run daemon without web UI:**
```bash
scripts/launch.sh --headless
```

Useful for servers or automated testing.

## References

- **Script Documentation**: `scripts/CLAUDE.md`
- **Steering Documents**: `.spec-workflow/specs/ai-dev-foundation/`
  - `requirements.md` - Detailed requirements
  - `design.md` - Architecture and design decisions
  - `tasks.md` - Implementation task breakdown
- **Project Structure**: `.spec-workflow/steering/structure.md`
- **CI/CD Workflows**: `.github/workflows/`
- **Rust API Guidelines**: https://rust-lang.github.io/api-guidelines/
