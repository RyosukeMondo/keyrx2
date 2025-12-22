# KeyRx UAT Guide - User Acceptance Testing

## Overview

This guide will help you test the KeyRx keyboard remapping daemon to verify all implemented features work correctly.

**Current Implementation Status:**
- ✅ Core runtime (event processing, state management)
- ✅ Configuration compiler (Rhai DSL to .krx binary)
- ✅ Config loader (load and validate .krx files)
- ✅ Event processor (orchestrate input → runtime → output)
- ✅ Mock platform (for testing without OS dependencies)
- ⚠️ Platform abstraction (traits defined, Linux/Windows not yet implemented)

**What Can Be Tested:**
- Configuration compilation from DSL to binary
- Config verification and validation
- Event processing pipeline (via mock devices)
- State management (modifiers, locks)
- Mapping types (simple, modifier, lock, conditional, modified output)

---

## Quick Start (5 minutes)

### 1. Build the Project

```bash
# Build all components
cargo build --release

# Verify build
ls -lh target/release/keyrx_compiler
```

**Expected output:**
```
-rwxr-xr-x 1 user user 8.2M Dec 22 17:30 target/release/keyrx_compiler
```

### 2. Create Test Configuration

```bash
# Create test directory
mkdir -p uat_tests

# Create sample config (see section below)
cat > uat_tests/basic.rhai <<'EOF'
device("*") {
    // Simple remap: A → B
    map(A, VK_B);

    // Escape → Caps Lock (swap)
    map(Esc, VK_CapsLock);
    map(CapsLock, VK_Esc);
}
EOF
```

### 3. Compile Configuration

```bash
# Compile to binary
./target/release/keyrx_compiler compile uat_tests/basic.rhai -o uat_tests/basic.krx

# Verify compilation
./target/release/keyrx_compiler verify uat_tests/basic.krx
```

**Expected output:**
```
✓ Valid .krx file
  Magic: KRX\n
  Version: 1.0.0
  Hash: 3a8f9c2e... (matches)
  Size: 1234 bytes
```

---

## Test Scenarios

### Test 1: Simple Key Remapping

**Objective:** Verify basic key mapping works

**Configuration:** `uat_tests/test1_simple.rhai`
```rhai
device("*") {
    map(A, VK_B);      // A → B
    map(B, VK_C);      // B → C
    map(C, VK_A);      // C → A (cycle)
}
```

**Steps:**
1. Compile: `./target/release/keyrx_compiler compile uat_tests/test1_simple.rhai -o uat_tests/test1.krx`
2. Verify: `./target/release/keyrx_compiler verify uat_tests/test1.krx`
3. Inspect: `./target/release/keyrx_compiler parse uat_tests/test1.krx`

**Expected Results:**
- Compilation succeeds
- Parse output shows 3 mappings
- Hash verification passes

**Pass Criteria:** ✅ All commands succeed without errors

---

### Test 2: Modifier Mappings

**Objective:** Test modifier key functionality

**Configuration:** `uat_tests/test2_modifiers.rhai`
```rhai
device("*") {
    // CapsLock becomes a layer modifier (MD_00)
    map(CapsLock, MD_00);

    // When CapsLock held: HJKL → Arrow keys
    when(MD_00) {
        map(H, VK_Left);
        map(J, VK_Down);
        map(K, VK_Up);
        map(L, VK_Right);
    }
}
```

**Steps:**
1. Compile and verify as above
2. Check parsed output shows conditional mappings

**Expected Parse Output:**
```json
{
  "devices": [{
    "identifier": { "pattern": "*" },
    "mappings": [
      { "Modifier": { "from": "CapsLock", "modifier_id": 0 } },
      { "Conditional": {
        "condition": { "ModifierActive": 0 },
        "mappings": [
          { "Simple": { "from": "H", "to": "Left" } },
          ...
        ]
      }}
    ]
  }]
}
```

**Pass Criteria:** ✅ Parse output matches expected structure

---

### Test 3: Lock Mappings

**Objective:** Test toggle lock functionality

**Configuration:** `uat_tests/test3_locks.rhai`
```rhai
device("*") {
    // ScrollLock toggles gaming mode (LK_00)
    map(ScrollLock, LK_00);

    // In gaming mode: Disable Windows key
    when_not(LK_00) {
        map(LWin, VK_LWin);  // Normal: pass through
    }

    when(LK_00) {
        // Gaming mode: suppress Windows key (no mapping = no output)
    }
}
```

**Steps:**
1. Compile and verify
2. Check for lock mapping and conditionals

**Pass Criteria:** ✅ Compilation succeeds, lock mapping present

---

### Test 4: Modified Output

**Objective:** Test chord/modified key output

**Configuration:** `uat_tests/test4_chords.rhai`
```rhai
device("*") {
    // Press 1 → Shift+1 (!)
    map(Num1, with_shift(VK_Num1));

    // Press 2 → Ctrl+C (copy)
    map(Num2, with_ctrl(VK_C));

    // Press 3 → Ctrl+Shift+Esc (Task Manager)
    map(Num3, with_mods(VK_Esc, shift: true, ctrl: true));
}
```

**Steps:**
1. Compile and verify
2. Parse output to verify ModifiedOutput mappings

**Expected Parse Output:**
```json
{
  "ModifiedOutput": {
    "from": "Num1",
    "to": "Num1",
    "with_shift": true,
    "with_ctrl": false,
    ...
  }
}
```

**Pass Criteria:** ✅ Parse shows correct modifier flags

---

### Test 5: Complex Multi-Layer

**Objective:** Test advanced layered configuration

**Configuration:** `uat_tests/test5_vim.rhai`
```rhai
device("*") {
    // CapsLock = Layer modifier
    map(CapsLock, MD_00);

    // Space = Layer modifier (only when held, not yet implemented tap/hold)
    map(Space, MD_01);

    // CapsLock layer: Vim navigation
    when(MD_00) {
        map(H, VK_Left);
        map(J, VK_Down);
        map(K, VK_Up);
        map(L, VK_Right);
        map(U, VK_PageUp);
        map(D, VK_PageDown);
    }

    // Space layer: Number row
    when(MD_01) {
        map(Q, VK_Num1);
        map(W, VK_Num2);
        map(E, VK_Num3);
        map(R, VK_Num4);
        map(T, VK_Num5);
    }

    // Both layers: Special functions
    when([MD_00, MD_01]) {
        map(Esc, VK_F12);  // Both held + Esc = F12
    }
}
```

**Steps:**
1. Compile and verify
2. Parse to verify nested conditionals
3. Check mapping count

**Expected:**
- 15+ mappings total
- Multiple conditional blocks
- AllActive condition for dual-layer

**Pass Criteria:** ✅ Complex config compiles without errors

---

### Test 6: Configuration Validation

**Objective:** Test error handling and validation

**Test Cases:**

#### 6a. Invalid Key Code
```rhai
device("*") {
    map(InvalidKey, VK_A);  // Should fail
}
```

**Expected:** ❌ Compilation error with helpful message

#### 6b. Out of Range Modifier ID
```rhai
device("*") {
    map(A, MD_FF);  // MD_255 is reserved
}
```

**Expected:** ❌ Error: "Modifier ID out of range (0-254)"

#### 6c. Invalid Prefix Usage
```rhai
device("*") {
    map(CapsLock, A);  // Should fail (no VK_ prefix)
}
```

**Expected:** ❌ Error: "Output key must have VK_, MD_, or LK_ prefix"

#### 6d. Circular Mapping (Allowed)
```rhai
device("*") {
    map(A, VK_B);
    map(B, VK_A);  // Allowed, processes sequentially
}
```

**Expected:** ✅ Compiles successfully (not a circular reference issue)

**Pass Criteria:** All error cases produce clear error messages

---

### Test 7: Hash Verification

**Objective:** Verify integrity checking works

**Steps:**

1. **Create valid config:**
```bash
./target/release/keyrx_compiler compile uat_tests/basic.rhai -o uat_tests/hash_test.krx
```

2. **Extract and verify hash:**
```bash
./target/release/keyrx_compiler hash uat_tests/hash_test.krx --extract
./target/release/keyrx_compiler hash uat_tests/hash_test.krx --verify
```

**Expected:** ✅ Hash matches

3. **Corrupt the file:**
```bash
# Backup original
cp uat_tests/hash_test.krx uat_tests/hash_test.krx.backup

# Corrupt byte 100
printf '\xFF' | dd of=uat_tests/hash_test.krx bs=1 seek=100 count=1 conv=notrunc

# Try to verify
./target/release/keyrx_compiler verify uat_tests/hash_test.krx
```

**Expected:** ❌ Error: "Hash mismatch (data corruption detected)"

**Pass Criteria:** Corruption is detected

---

### Test 8: Deterministic Compilation

**Objective:** Verify same input always produces same output

**Steps:**

1. **Compile twice:**
```bash
./target/release/keyrx_compiler compile uat_tests/basic.rhai -o uat_tests/det1.krx
./target/release/keyrx_compiler compile uat_tests/basic.rhai -o uat_tests/det2.krx
```

2. **Compare hashes:**
```bash
sha256sum uat_tests/det1.krx uat_tests/det2.krx
```

**Expected:** Hashes are identical (excluding timestamp metadata)

**Note:** Currently timestamps are embedded, so files differ. This is expected for now.

**Pass Criteria:** ⚠️ Known limitation - timestamps cause difference

---

### Test 9: Multiple Devices

**Objective:** Test device-specific configurations

**Configuration:** `uat_tests/test9_multidevice.rhai`
```rhai
// Laptop keyboard
device("AT Translated Set 2 keyboard") {
    // Map right Alt to Ctrl (for laptops without right Ctrl)
    map(RAlt, VK_RCtrl);
}

// External mechanical keyboard
device("Keychron K2") {
    // No remapping needed, use as-is
}

// Catch-all for other devices
device("*") {
    // Basic swap for all other keyboards
    map(CapsLock, VK_Esc);
}
```

**Steps:**
1. Compile and parse
2. Verify 3 device blocks exist
3. Check pattern matching

**Pass Criteria:** ✅ Parse shows 3 separate device configs

---

### Test 10: CLI Functionality

**Objective:** Test all compiler commands

**Commands to test:**

1. **Help:**
```bash
./target/release/keyrx_compiler --help
./target/release/keyrx_compiler compile --help
./target/release/keyrx_compiler parse --help
./target/release/keyrx_compiler verify --help
./target/release/keyrx_compiler hash --help
```

2. **Parse (human-readable):**
```bash
./target/release/keyrx_compiler parse uat_tests/test1.krx
```

3. **Parse (JSON):**
```bash
./target/release/keyrx_compiler parse uat_tests/test1.krx --json
```

4. **Verify:**
```bash
./target/release/keyrx_compiler verify uat_tests/test1.krx
```

5. **Hash operations:**
```bash
./target/release/keyrx_compiler hash uat_tests/test1.krx --extract
./target/release/keyrx_compiler hash uat_tests/test1.krx --verify
```

**Pass Criteria:** All commands execute without errors

---

## Integration Testing (Mock Platform)

**Note:** Linux/Windows platform implementations are not yet complete. We can test the event processing pipeline using mock devices in unit tests, but not with real keyboard input yet.

### What's Implemented:
- ✅ Event processor orchestration
- ✅ Mock input/output devices (for testing)
- ✅ Complete runtime logic

### What's Not Yet Implemented:
- ⚠️ Linux evdev/uinput integration (planned)
- ⚠️ Windows low-level hooks (planned)
- ⚠️ Daemon service/background mode (planned)

### Test Available Functionality:

Run unit and integration tests:
```bash
# Run all tests
cargo test --workspace

# Run integration tests only
cargo test --test integration_tests

# Run processor tests
cargo test -p keyrx_daemon processor
```

**Expected Results:**
- ✅ 353 tests pass
- ✅ No failures
- ✅ Coverage >76%

---

## UAT Checklist

Use this checklist to track your testing progress:

### Compiler Functionality
- [ ] Test 1: Simple remapping compiles
- [ ] Test 2: Modifier mappings compile
- [ ] Test 3: Lock mappings compile
- [ ] Test 4: Modified output compiles
- [ ] Test 5: Complex multi-layer compiles
- [ ] Test 6a-d: Error validation works
- [ ] Test 7: Hash verification detects corruption
- [ ] Test 8: Deterministic compilation (timestamps differ, expected)
- [ ] Test 9: Multiple devices compile
- [ ] Test 10: All CLI commands work

### Configuration Validation
- [ ] Invalid key codes are rejected
- [ ] Out of range IDs are rejected
- [ ] Missing prefixes are caught
- [ ] Clear error messages displayed

### File Operations
- [ ] .krx files are created successfully
- [ ] File sizes are reasonable (<10KB for typical configs)
- [ ] Verify command validates correctly
- [ ] Parse command outputs valid JSON

### Quality
- [ ] All unit tests pass (cargo test)
- [ ] All integration tests pass
- [ ] No clippy warnings
- [ ] Code is formatted

---

## Known Limitations (Expected for Current Phase)

1. **No Real Platform Support Yet:**
   - Can compile configs
   - Can verify configs
   - Cannot intercept real keyboard input yet
   - Mock platform available for testing

2. **No Tap/Hold Yet:**
   - Planned for future implementation
   - Current focus: static mappings

3. **No Daemon Mode Yet:**
   - Planned for future
   - Currently: compile and test only

4. **Timestamps in Binary:**
   - Causes non-deterministic compilation
   - This is acceptable for now

---

## Success Criteria

**UAT PASSES if:**
- ✅ All 10 test scenarios compile without errors
- ✅ Configuration validation catches all error cases
- ✅ Hash verification detects corruption
- ✅ Parse output is valid and readable
- ✅ All unit/integration tests pass
- ✅ CLI commands work as documented

**UAT FAILS if:**
- ❌ Any valid config fails to compile
- ❌ Invalid configs are accepted without errors
- ❌ Corruption goes undetected
- ❌ Tests fail
- ❌ CLI crashes or hangs

---

## Troubleshooting

### "Command not found: keyrx_compiler"
**Solution:** Build with `cargo build --release` first

### "Permission denied"
**Solution:** `chmod +x target/release/keyrx_compiler`

### "Invalid magic bytes"
**Solution:** File is corrupted or not a .krx file. Recompile.

### "Hash mismatch"
**Solution:** File was modified after compilation. This is expected for Test 7.

### Tests failing
**Solution:** Run `cargo test -- --nocapture` to see detailed output

---

## Next Steps After UAT

Once UAT passes:

1. **Linux Platform Implementation**
   - Implement evdev input capture
   - Implement uinput output injection
   - Test with real keyboard

2. **Daemon Mode**
   - Systemd service file
   - Background execution
   - Auto-load config on startup

3. **Advanced Features**
   - Tap/hold detection
   - Timeout-based actions
   - Web UI for live monitoring

---

## Support

If you encounter issues during UAT:

1. Check logs: `cargo test -- --nocapture`
2. Review error messages (designed to be helpful)
3. Verify file permissions
4. Ensure Rust 1.70+ is installed
5. Try `cargo clean && cargo build --release`

**UAT Complete!** Proceed with production deployment once all tests pass.
