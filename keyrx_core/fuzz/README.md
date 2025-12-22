# Fuzzing Infrastructure for keyrx_core

This directory contains fuzz targets for testing keyrx_core components with libFuzzer.

## Overview

Fuzzing is used to discover edge cases, crashes, and undefined behavior by feeding components with randomly generated inputs. This helps ensure reliability and security.

## Fuzz Targets

### 1. `fuzz_deserialize` - Binary Deserializer Fuzzing
**Target**: `keyrx_compiler::serialize::deserialize()`
**Purpose**: Test .krx binary deserialization with malformed inputs
**Results**: [FUZZING_RESULTS.md](./FUZZING_RESULTS.md)

**Findings**:
- ✅ 2 issues fixed (empty data, undersized archives)
- ⚠️ 1 known limitation (rkyv internal structure validation)

**Status**: ✅ Complete

---

### 2. `fuzz_runtime` - Runtime Event Processing Fuzzing
**Target**: Runtime event processing pipeline
**Components**:
- `keyrx_core::runtime::event::process_event()`
- `keyrx_core::runtime::lookup::KeyLookup`
- `keyrx_core::runtime::state::DeviceState`

**Purpose**: Verify event processing handles arbitrary inputs without panics or infinite loops
**Results**: [RUNTIME_FUZZING_RESULTS.md](./RUNTIME_FUZZING_RESULTS.md)

**Findings**:
- ✅ 199,663 test cases executed without crashes
- ✅ No panics, no infinite loops
- ✅ Event amplification verified as expected behavior

**Status**: ✅ Complete

---

### 3. `fuzz_parser` - Rhai Parser Fuzzing
**Target**: Rhai configuration parser
**Purpose**: Test Rhai script parsing with malformed inputs
**Status**: ⚠️ Basic setup (needs comprehensive testing)

---

## Running Fuzz Tests

### Prerequisites

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Fuzzing requires nightly Rust
rustup toolchain install nightly
```

### Quick Start

```bash
# Run all fuzz targets for 60 seconds each
make fuzz-all

# Or run individually:
cargo +nightly fuzz run fuzz_deserialize -- -max_total_time=60
cargo +nightly fuzz run fuzz_runtime -- -max_total_time=60
cargo +nightly fuzz run fuzz_parser -- -max_total_time=60
```

### Common Commands

```bash
# List available fuzz targets
cargo +nightly fuzz list

# Run specific target indefinitely (Ctrl+C to stop)
cargo +nightly fuzz run <target>

# Run with custom time limit (in seconds)
cargo +nightly fuzz run <target> -- -max_total_time=300

# Run with execution count limit
cargo +nightly fuzz run <target> -- -runs=1000000

# Reproduce a crash
cargo +nightly fuzz run <target> artifacts/<target>/crash-<hash>

# Minimize a crash case
cargo +nightly fuzz tmin <target> artifacts/<target>/crash-<hash>

# Build without running
cargo +nightly fuzz build <target>
```

## Directory Structure

```
fuzz/
├── Cargo.toml                   # Fuzz harness dependencies
├── README.md                    # This file
├── FUZZING_RESULTS.md           # Deserializer fuzzing results
├── RUNTIME_FUZZING_RESULTS.md   # Runtime fuzzing results
├── fuzz_targets/                # Fuzz target implementations
│   ├── fuzz_deserialize.rs      # Binary deserializer fuzzing
│   ├── fuzz_runtime.rs          # Runtime event processing fuzzing
│   ├── fuzz_parser.rs           # Rhai parser fuzzing
│   └── fuzz_target_1.rs         # Legacy/example target
├── corpus/                      # Interesting test cases (auto-generated)
│   ├── fuzz_deserialize/
│   ├── fuzz_runtime/
│   └── fuzz_parser/
└── artifacts/                   # Crash cases and minimized inputs
    ├── fuzz_deserialize/
    ├── fuzz_runtime/
    └── fuzz_parser/
```

## Interpreting Results

### Success

```
INFO: Running with entropic power schedule (0xFF, 100).
#100000 NEW cov: 294 ft: 1029 corp: 131/16kB lim: 1200 exec/s: 3333 rss: 591Mb
```

- `cov`: Code coverage (branches explored)
- `ft`: Features (distinct behaviors)
- `corp`: Corpus size (interesting inputs found)
- `exec/s`: Executions per second
- `rss`: Memory usage

### Crash Found

```
==12345== ERROR: libFuzzer: deadly signal
SUMMARY: libFuzzer: deadly signal
artifact_prefix='artifacts/'; Test unit written to artifacts/crash-<hash>
```

**Action**: Investigate the crash:
1. Check the results markdown file for analysis
2. Reproduce: `cargo +nightly fuzz run <target> artifacts/<target>/crash-<hash>`
3. Minimize: `cargo +nightly fuzz tmin <target> artifacts/<target>/crash-<hash>`
4. Fix the underlying issue
5. Re-run fuzzer to verify fix

## CI Integration

Fuzzing is integrated into CI via GitHub Actions (future):

```yaml
# .github/workflows/fuzz.yml
- name: Run fuzzing for 5 minutes
  run: |
    cargo +nightly fuzz run fuzz_deserialize -- -max_total_time=300
    cargo +nightly fuzz run fuzz_runtime -- -max_total_time=300
```

## Performance Tips

1. **Increase time for deeper testing**:
   ```bash
   cargo +nightly fuzz run <target> -- -max_total_time=3600  # 1 hour
   ```

2. **Use multiple cores**:
   ```bash
   cargo +nightly fuzz run <target> -- -jobs=8
   ```

3. **Merge corpus from multiple runs**:
   ```bash
   cargo +nightly fuzz run <target> corpus/<target> other_corpus/
   ```

4. **Clean start (remove existing corpus)**:
   ```bash
   rm -rf corpus/<target>/*
   cargo +nightly fuzz run <target>
   ```

## Maintenance

### Adding a New Fuzz Target

1. Create fuzz target:
   ```bash
   cargo +nightly fuzz add fuzz_new_feature
   ```

2. Edit `fuzz_targets/fuzz_new_feature.rs`:
   ```rust
   #![no_main]
   use libfuzzer_sys::fuzz_target;

   fuzz_target!(|data: &[u8]| {
       // Your fuzzing logic here
       let _ = your_function(data);
   });
   ```

3. Run it:
   ```bash
   cargo +nightly fuzz run fuzz_new_feature
   ```

4. Document results in `FEATURE_FUZZING_RESULTS.md`

### Updating Existing Targets

1. Edit `fuzz_targets/<target>.rs`
2. Test locally: `cargo +nightly fuzz build <target>`
3. Run for verification: `cargo +nightly fuzz run <target> -- -max_total_time=60`
4. Update corresponding results markdown file

## Troubleshooting

### Error: "nightly toolchain required"

```bash
rustup toolchain install nightly
```

### Error: "cargo-fuzz not found"

```bash
cargo install cargo-fuzz
```

### Fuzzer runs out of memory

Reduce corpus size or use memory limits:
```bash
cargo +nightly fuzz run <target> -- -rss_limit_mb=2048
```

### Fuzzer stuck (no new coverage)

This is normal after initial exploration. Let it run longer or try:
```bash
# Restart with fresh corpus
rm -rf corpus/<target>/*
cargo +nightly fuzz run <target>
```

## References

- [cargo-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Rust Fuzz project](https://github.com/rust-fuzz)
- Task 25: Add fuzzing infrastructure (basic-key-remapping spec)
