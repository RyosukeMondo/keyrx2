# Unwrap Handling Strategy - Industry Standard Approach

## Current Situation

**Current approach:** Manual counting of unwraps using `scripts/check_unwraps.sh`
- **Threshold:** 410 unwraps maximum
- **Problem:** Fragile, requires manual threshold updates, doesn't distinguish between safe and unsafe unwraps

**Current count:** 405/410 unwraps (passing)

## Industry-Standard Solutions

### Option 1: Clippy Lints (RECOMMENDED - Zero-Cost)

**Advantages:**
- ✅ Built into Rust toolchain (no external dependencies)
- ✅ Enforces best practices automatically
- ✅ Can be configured per-file or per-function with `#[allow]` attributes
- ✅ Catches issues at compile time
- ✅ More maintainable than manual counting

**Implementation:**

Add to `Cargo.toml` workspace section:

```toml
[workspace.lints.clippy]
# Deny unwrap in production code
unwrap_used = "warn"  # Start with warn, upgrade to "deny" later
expect_used = "warn"  # Prefer proper error propagation
panic = "warn"        # Avoid panics in production
indexing_slicing = "warn"  # Avoid index panics

# Allow in test code
[lints.clippy]
unwrap_used = { level = "allow", priority = 1, cfg = 'test' }
```

**Migration path:**
1. Start with "warn" level to see all violations
2. Fix critical unwraps in stages
3. Use `#[allow(clippy::unwrap_used)]` for proven-safe cases with SAFETY comments
4. Gradually upgrade to "deny" once count is reduced

### Option 2: anyhow + thiserror (Industry Standard Error Handling)

**Advantages:**
- ✅ Industry standard for Rust error handling
- ✅ Better error messages with context
- ✅ Easier error propagation with `?` operator
- ✅ Type-safe error handling
- ✅ Used by major projects (tokio, axum, serde, etc.)

**Dependencies to add:**

```toml
[workspace.dependencies]
anyhow = "1.0"      # For application code (keyrx_daemon)
thiserror = "2.0"   # For library code (keyrx_core, keyrx_compiler)
```

**Usage pattern:**

```rust
// Before: unwrap() - panics on error
let config = fs::read_to_string(path).unwrap();

// After: anyhow with context - proper error handling
use anyhow::{Context, Result};

fn load_config(path: &str) -> Result<Config> {
    let content = fs::read_to_string(path)
        .context("Failed to read config file")?;

    let config: Config = serde_json::from_str(&content)
        .context("Failed to parse config JSON")?;

    Ok(config)
}
```

**Custom error types with thiserror:**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProfileError {
    #[error("Profile not found: {0}")]
    NotFound(String),

    #[error("Invalid profile configuration")]
    InvalidConfig,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Option 3: cargo-deny (Policy Enforcement)

**Advantages:**
- ✅ Enforces project-wide policies
- ✅ Checks dependencies, licenses, and advisories
- ✅ Can be integrated into CI/CD
- ✅ Supports custom deny rules

**Installation:**
```bash
cargo install cargo-deny
```

**Configuration in `deny.toml`:**
```toml
[advisories]
db-path = "~/.cargo/advisory-db"
vulnerability = "deny"

[sources]
unknown-registry = "deny"

[bans]
multiple-versions = "warn"
```

## Recommended Migration Plan

### Phase 1: Enable Clippy Lints (Week 1) - IMMEDIATE

1. **Add clippy lints to workspace Cargo.toml:**
   ```toml
   [workspace.lints.clippy]
   unwrap_used = "warn"
   expect_used = "warn"
   ```

2. **Run clippy to audit all unwraps:**
   ```bash
   cargo clippy --workspace 2>&1 | grep unwrap > unwrap_audit.txt
   ```

3. **Keep current unwrap count check** as a backup during migration

### Phase 2: Add Error Handling Libraries (Week 2-3)

1. **Add dependencies:**
   - `anyhow` for keyrx_daemon (application code)
   - `thiserror` for keyrx_core and keyrx_compiler (library code)

2. **Migrate high-priority modules:**
   - Start with public APIs
   - Focus on CLI handlers
   - Update configuration loading

3. **Define custom error types** using thiserror

### Phase 3: Gradual Unwrap Reduction (Weeks 4-8)

1. **Fix unwraps in batches:**
   - Week 4: CLI modules (30-40 unwraps)
   - Week 5: Config modules (40-50 unwraps)
   - Week 6: Platform modules (30-40 unwraps)
   - Week 7: Web handlers (20-30 unwraps)
   - Week 8: Remaining modules (remaining unwraps)

2. **Document legitimate unwraps:**
   - Add `SAFETY:` comments explaining why unwrap is safe
   - Use `#[allow(clippy::unwrap_used)]` with justification

### Phase 4: Lock Down Quality Gates (Week 9+)

1. **Upgrade clippy lints from "warn" to "deny":**
   ```toml
   unwrap_used = "deny"
   ```

2. **Remove manual unwrap count check** (replaced by clippy)

3. **Add to CI/CD pipeline:**
   ```yaml
   - name: Check clippy lints
     run: cargo clippy --workspace -- -D warnings
   ```

## Legitimate Unwrap Cases (with SAFETY comments)

Some unwraps are safe and acceptable:

```rust
// ✅ SAFE: Mutex poisoning is unrecoverable
let guard = mutex.lock().unwrap();

// ✅ SAFE: Static regex known to be valid at compile time
static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d+$").unwrap()  // SAFETY: regex is known valid
});

// ✅ SAFE: Array index within bounds by construction
let first = [1, 2, 3][0];  // SAFETY: array is non-empty

// ✅ SAFE: Test code (tests are allowed to panic)
#[cfg(test)]
#[test]
fn test_parsing() {
    let result = parse("test").unwrap();  // OK in tests
    assert_eq!(result, expected);
}
```

## Examples from Industry Leaders

**tokio (async runtime):**
- Uses `thiserror` for error types
- Uses `anyhow` in examples
- Zero unwraps in production code

**serde (serialization):**
- Uses `thiserror` for error definitions
- Propagates errors with `?` operator
- Clear error messages with context

**axum (web framework):**
- Uses `anyhow` for application errors
- Custom error types with `thiserror`
- Implements `IntoResponse` for error types

## References

### Official Documentation
- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [anyhow documentation](https://docs.rs/anyhow/)
- [thiserror documentation](https://docs.rs/thiserror/)
- [Clippy lints reference](https://rust-lang.github.io/rust-clippy/master/)

### Best Practices Articles
- [Using unwrap() in Rust is Okay](https://burntsushi.net/unwrap/) - Andrew Gallant
- [Don't Unwrap Options: Better Ways](https://corrode.dev/blog/rust-option-handling-best-practices/)
- [Error Handling in Rust: From unwrap to Best Practices](https://medium.com/@adamszpilewicz/error-handling-in-rust-from-unwrap-to-best-practices-163f49e48898)

## Decision

**Recommended approach:** Phase 1 implementation IMMEDIATELY
- Low risk, high reward
- No code changes required initially
- Provides visibility into unwrap usage
- Establishes foundation for gradual improvement

**Next steps:**
1. Add clippy lints to workspace Cargo.toml (5 minutes)
2. Run audit to see all unwraps (2 minutes)
3. Plan migration for high-priority modules (30 minutes)
4. Keep existing unwrap count check as backup (no changes)

This approach is:
- ✅ Industry standard
- ✅ Zero-cost abstraction
- ✅ Maintainable long-term
- ✅ Gradually adoptable
- ✅ Used by major Rust projects
