# RFC: Keyboard Internationalization

## Problem Statement

keyrx currently has ~100 hardcoded `KeyCode` variants covering US keyboard layout. Users with regional keyboards (Japanese JIS, German QWERTZ, French AZERTY, Korean, etc.) cannot map their locale-specific keys.

### Missing Key Examples

| Region | Keys |
|--------|------|
| Japanese (JIS) | Kanji, Eisuu, Hiragana, Katakana, Muhenkan, Henkan, Yen, Ro |
| German (QWERTZ) | Umlaut keys (Ä, Ö, Ü, ß), AltGr combinations |
| Korean | Hangul, Hanja |
| Nordic | Ø, Å, Æ |
| Brazilian (ABNT2) | Cedilla (Ç), extra keys |

---

## Option A: Hybrid Enum + Scancode (Recommended)

### Design

```rust
// keyrx_core/src/config/keys.rs
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum KeyCode {
    // === Standard Keys (type-safe, ~150 variants) ===
    A = 0x00, B = 0x01, /* ... */ Z = 0x19,
    Num0 = 0x20, /* ... */ Num9 = 0x29,
    F1 = 0x30, /* ... */ F24 = 0x47,
    // Modifiers, arrows, special keys...

    // === Extended Key (scancode-based) ===
    /// Extended key identified by raw scancode
    /// Format: 0xE000 | scancode (top 4 bits = 0xE for "extended")
    Extended(u16) = 0xE000,
}
```

### Rhai DSL Usage

```rhai
// Standard keys - type-safe
map("VK_A", "VK_B");

// Extended keys by scancode (Linux evdev codes)
map("SC_124", "SC_125");  // Map scancode 124 to 125

// Named aliases loaded from config
map("VK_Kanji", "VK_Eisuu");  // If alias config loaded
```

### Key Alias Config (TOML)

```toml
# ~/.config/keyrx/key_aliases.toml
[japanese]
Kanji = 0x7A       # KEY_ZENKAKUHANKAKU
Eisuu = 0x7B       # KEY_KATAKANA
Hiragana = 0x77    # KEY_HIRAGANA
Muhenkan = 0x5E    # KEY_MUHENKAN
Henkan = 0x5C      # KEY_HENKAN
Yen = 0x7D         # KEY_YEN
Ro = 0x59          # KEY_RO

[german]
SZ = 0x0C          # ß key
AE = 0x28          # Ä key
OE = 0x27          # Ö key
UE = 0x1A          # Ü key
```

### Pros
- Type-safe for common keys (compile-time errors)
- Extensible via config without recompilation
- Users can add ANY key via scancode
- Works with any keyboard layout
- no_std compatible (Extended is just a u16 wrapper)

### Cons
- Slight complexity in key parsing
- Users need to know scancodes for truly custom keys
- Two syntaxes (VK_* vs SC_*)

### Implementation Effort: Medium (~3-5 tasks)

---

## Option B: Full Config-Driven Keys

### Design

All key names defined in external config, minimal hardcoded keys.

```rust
// keyrx_core/src/config/keys.rs
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyCode(pub u16);  // Just a wrapper around scancode

impl KeyCode {
    pub const fn from_scancode(sc: u16) -> Self { Self(sc) }
}
```

### Key Definition Config (TOML)

```toml
# /etc/keyrx/keys.toml (system-wide)
[keys]
A = 0x1E
B = 0x30
# ... all standard keys defined here

# ~/.config/keyrx/keys.toml (user override/extension)
[keys]
Kanji = 0x7A
Eisuu = 0x7B
```

### Rhai Usage

```rhai
map("A", "B");           // Looked up from config
map("Kanji", "Eisuu");   // Also from config
map(0x7A, 0x7B);         // Raw scancode always works
```

### Pros
- Maximum flexibility - any key layout supported
- Community can share key definition files
- No code changes needed for new layouts
- Simple core implementation

### Cons
- No compile-time type safety
- Requires config file to exist
- Runtime parsing overhead
- Error messages less helpful
- Breaks no_std (needs file I/O)

### Implementation Effort: Medium-High (~5-7 tasks)

---

## Option C: Expanded Enum (Hardcode Everything)

### Design

Add all regional keys directly to the enum.

```rust
pub enum KeyCode {
    // Standard keys...
    A, B, C, /* ... */

    // Japanese JIS
    Kanji, Eisuu, Hiragana, Katakana, Muhenkan, Henkan, Yen, Ro,

    // German
    SZ, Umlaut_A, Umlaut_O, Umlaut_U,

    // Korean
    Hangul, Hanja,

    // Nordic
    AE_Ligature, O_Stroke, A_Ring,

    // ... hundreds more
}
```

### Pros
- Full type safety
- Best IDE autocomplete
- Compile-time validation
- Fastest runtime (no lookup)

### Cons
- Enum becomes huge (~500+ variants)
- Requires code change for new keys
- Longer compile times
- Complex maintenance

### Implementation Effort: Low initially, High ongoing

---

## Option D: Plugin System

### Design

Allow dynamic loading of key definition modules.

```rust
// Plugin trait
pub trait KeyPlugin {
    fn name(&self) -> &str;
    fn resolve_name(&self, name: &str) -> Option<u16>;
    fn resolve_scancode(&self, sc: u16) -> Option<&str>;
}

// Japanese plugin
struct JapaneseKeys;
impl KeyPlugin for JapaneseKeys {
    fn resolve_name(&self, name: &str) -> Option<u16> {
        match name {
            "Kanji" => Some(0x7A),
            "Eisuu" => Some(0x7B),
            _ => None,
        }
    }
}
```

### Pros
- Most extensible
- Clean separation of concerns
- Community plugins

### Cons
- Most complex to implement
- Dynamic dispatch overhead
- Plugin management complexity
- Not no_std compatible

### Implementation Effort: High (~10+ tasks)

---

## Recommendation: Option A (Hybrid)

| Criteria | A (Hybrid) | B (Config) | C (Hardcode) | D (Plugin) |
|----------|------------|------------|--------------|------------|
| Type Safety | ✅ Partial | ❌ None | ✅ Full | ⚠️ Partial |
| Extensibility | ✅ Good | ✅ Best | ❌ Poor | ✅ Best |
| no_std | ✅ Yes | ❌ No | ✅ Yes | ❌ No |
| Complexity | ⚠️ Medium | ⚠️ Medium | ✅ Low | ❌ High |
| Maintenance | ✅ Low | ⚠️ Medium | ❌ High | ⚠️ Medium |
| User Experience | ✅ Good | ⚠️ Config needed | ✅ Best | ⚠️ Plugin needed |

### Why Option A?

1. **Pragmatic balance**: Common keys are type-safe, regional keys are extensible
2. **no_std compatible**: Critical for WASM/embedded targets
3. **Immediate value**: Users can use scancodes TODAY, aliases are optional
4. **Low maintenance**: Community maintains alias configs, not code
5. **Backward compatible**: Existing configs keep working

---

## Implementation Plan (Option A)

### Phase 1: Scancode Support (Immediate)
1. Add `Extended(u16)` variant to KeyCode
2. Add `SC_xxx` parsing to compiler
3. Update evdev mapping to handle extended keys
4. Add documentation

### Phase 2: Alias Config (Optional Enhancement)
1. Define TOML schema for key aliases
2. Add config loading to compiler
3. Create Japanese, German, Korean alias files
4. Add `--key-aliases` compiler flag

### Phase 3: Community (Future)
1. Repository for community key configs
2. Documentation for adding new layouts
3. Key discovery tool (show scancode on keypress)

---

## Questions for Decision

1. **Which option do you prefer?** (A/B/C/D)
2. **Should aliases be per-config or global?** (per-Rhai-file import vs ~/.config)
3. **Scancode format preference?** (`SC_124` vs `0x7C` vs `KEY_ZENKAKUHANKAKU`)
4. **Priority?** (Add scancode support first, then aliases?)
