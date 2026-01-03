# Error Handling Guide

## Overview

This guide documents the keyrx2 error handling architecture following the migration from ad-hoc `unwrap()` usage to industry-standard error handling using `anyhow` and `thiserror`.

**Key principles:**
- Use `thiserror` for library code (keyrx_core, keyrx_compiler) where type-safe custom errors are needed
- Use `anyhow` for application code (keyrx_daemon) where context-rich errors are needed
- Propagate errors with the `?` operator instead of using `unwrap()`
- Enforce error handling best practices through clippy lints (unwrap_used = "deny")

## Error Type Hierarchy

### Architecture Overview

```
std::error::Error
├── DaemonError (keyrx_daemon top-level)
│   ├── ConfigError (configuration errors)
│   ├── PlatformError (platform-specific errors)
│   ├── WebError (web server errors)
│   ├── CliError (CLI-specific errors)
│   ├── CoreError (from keyrx_core)
│   └── Io(std::io::Error)
│
├── CoreError (keyrx_core library)
│   ├── InvalidState
│   ├── Validation
│   └── Config
│
└── CompilerError (keyrx_compiler)
    ├── ParseError
    └── CompileError
```

### DaemonError (keyrx_daemon/src/error.rs)

Top-level error type for the daemon application.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DaemonError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Platform error: {0}")]
    Platform(#[from] PlatformError),

    #[error("Web server error: {0}")]
    Web(String),

    #[error("CLI error: {0}")]
    Cli(String),

    #[error("Core error: {0}")]
    Core(#[from] keyrx_core::CoreError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type DaemonResult<T> = Result<T, DaemonError>;
```

**Usage:**
```rust
use crate::error::{DaemonError, DaemonResult};

fn load_configuration(path: &Path) -> DaemonResult<Config> {
    let content = std::fs::read_to_string(path)?; // Automatically converts io::Error
    let config = serde_json::from_str(&content)?; // Automatically converts serde_json::Error
    Ok(config)
}
```

### ConfigError (keyrx_daemon/src/error.rs)

Configuration-related errors with rich context.

```rust
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },

    #[error("Failed to parse configuration: {0}")]
    Parse(String),

    #[error("Invalid profile '{name}': {reason}")]
    InvalidProfile { name: String, reason: String },

    #[error("Profile compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Profile error: {0}")]
    Profile(#[from] ProfileError),

    #[error("Generator error: {0}")]
    Generator(#[from] GeneratorError),
}
```

**Usage:**
```rust
fn load_profile(name: &str) -> Result<Profile, ConfigError> {
    if !profile_exists(name) {
        return Err(ConfigError::InvalidProfile {
            name: name.to_string(),
            reason: "Profile does not exist".to_string(),
        });
    }
    // ...
}
```

### PlatformError (keyrx_daemon/src/error.rs)

Platform-specific operation errors with user-friendly suggestions.

```rust
#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Failed to access device {device}: {reason}. {suggestion}")]
    DeviceAccess {
        device: String,
        reason: String,
        suggestion: String,
    },

    #[error("Keyboard injection failed: {0}")]
    InjectionFailed(String),

    #[error("Unsupported platform operation: {0}")]
    Unsupported(String),

    #[error("Platform initialization failed: {0}")]
    InitializationFailed(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

**Usage:**
```rust
fn open_device(path: &str) -> Result<Device, PlatformError> {
    Device::open(path).map_err(|e| {
        PlatformError::DeviceAccess {
            device: path.to_string(),
            reason: e.to_string(),
            suggestion: "Run with sudo or add your user to the 'input' group".to_string(),
        }
    })
}
```

### CoreError (keyrx_core/src/error.rs)

Base error type for the core library (no_std compatible).

```rust
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Invalid state transition: {0}")]
    InvalidState(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type CoreResult<T> = Result<T, CoreError>;
```

## Error Propagation Patterns

### Basic Propagation with ?

Replace `unwrap()` with `?` operator:

```rust
// ❌ Before: Panics on error
fn load_config(path: &str) -> Config {
    let content = fs::read_to_string(path).unwrap();
    serde_json::from_str(&content).unwrap()
}

// ✅ After: Propagates errors
fn load_config(path: &str) -> DaemonResult<Config> {
    let content = fs::read_to_string(path)?;
    let config = serde_json::from_str(&content)?;
    Ok(config)
}
```

### Error Conversion with #[from]

Automatic error conversion using `thiserror`'s `#[from]` attribute:

```rust
#[derive(Error, Debug)]
pub enum DaemonError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),  // Automatic From<io::Error> implementation
}

// Now io::Error automatically converts to DaemonError
fn read_file(path: &str) -> DaemonResult<String> {
    Ok(std::fs::read_to_string(path)?) // io::Error converts automatically
}
```

### Adding Context with Custom Messages

Add context to errors for better debugging:

```rust
use crate::error::DaemonError;

fn load_profile(name: &str) -> DaemonResult<Profile> {
    let path = get_profile_path(name)?;

    let content = std::fs::read_to_string(&path)
        .map_err(|e| DaemonError::Config(ConfigError::FileNotFound {
            path: path.display().to_string(),
        }))?;

    let profile: Profile = serde_json::from_str(&content)
        .map_err(|e| DaemonError::Config(ConfigError::Parse(
            format!("Invalid JSON in profile '{}': {}", name, e)
        )))?;

    Ok(profile)
}
```

### Error Recovery

Handle errors gracefully instead of crashing:

```rust
// ❌ Before: Crashes on timestamp error
let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_micros() as i64;

// ✅ After: Falls back to 0 with warning
let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_micros() as i64)
    .unwrap_or_else(|e| {
        log::warn!("Failed to get timestamp: {}, using 0", e);
        0
    });
```

## CLI Error Formatting

### Human-Readable Output

Errors are automatically formatted for CLI output:

```rust
use crate::error::DaemonError;

// Error propagates to main
fn run_command(args: &Args) -> DaemonResult<()> {
    let config = load_config(&args.config_path)?;
    // ... more operations
    Ok(())
}

// Main catches and formats errors
fn main() {
    if let Err(e) = run_command(&args) {
        eprintln!("Error: {}", e);

        // Print error chain
        let mut source = e.source();
        while let Some(err) = source {
            eprintln!("  Caused by: {}", err);
            source = err.source();
        }

        std::process::exit(1);
    }
}
```

### JSON Output

Errors can be formatted as JSON for programmatic consumption:

```rust
use serde_json::json;

fn format_error_json(error: &DaemonError) -> String {
    json!({
        "error": error.to_string(),
        "type": error_type_name(error),
    }).to_string()
}
```

## Web API Error Responses

### IntoResponse Implementation

Errors automatically convert to HTTP responses:

```rust
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use axum::Json;

impl IntoResponse for DaemonError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            DaemonError::Config(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            DaemonError::Platform(_) | DaemonError::Io(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

### Handler Example

Web handlers return Result types that automatically convert to HTTP responses:

```rust
use axum::{Json, extract::Path};

async fn get_profile(
    Path(name): Path<String>,
) -> Result<Json<Profile>, DaemonError> {
    let profile = load_profile(&name)?; // ConfigError converts to 400 Bad Request
    Ok(Json(profile))
}
```

## Legitimate Unwrap Cases

Some unwraps are safe and acceptable with proper documentation:

### Mutex Lock Unwrap

```rust
// SAFETY: Mutex cannot be poisoned because no panic paths exist while lock is held
#[allow(clippy::unwrap_used)]
let guard = self.state.lock().unwrap();
```

### SystemTime UNIX_EPOCH

```rust
// SAFETY: SystemTime is always after UNIX_EPOCH on modern systems
#[allow(clippy::unwrap_used)]
let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_micros() as i64;
```

### Response Builder with Valid StatusCode

```rust
// SAFETY: Response builder cannot fail with valid StatusCode
#[allow(clippy::unwrap_used)]
let response = Response::builder()
    .status(StatusCode::OK)
    .body(body)
    .unwrap();
```

### Control Flow Guarantees

```rust
fn process_optional(value: Option<i32>) -> i32 {
    if value.is_none() {
        return 0;
    }

    // SAFETY: Value is guaranteed to be Some() by the check above
    #[allow(clippy::unwrap_used)]
    value.unwrap()
}
```

### Static Regex Compilation

```rust
use once_cell::sync::Lazy;
use regex::Regex;

// SAFETY: Regex pattern is known to be valid at compile time
static KEY_REGEX: Lazy<Regex> = Lazy::new(|| {
    #[allow(clippy::unwrap_used)]
    Regex::new(r"^[A-Z][a-z]+$").unwrap()
});
```

## Common Migration Patterns

### Pattern 1: File Operations

```rust
// ❌ Before
fn read_config() -> Config {
    let path = "/etc/keyrx/config.json";
    let content = fs::read_to_string(path).unwrap();
    serde_json::from_str(&content).unwrap()
}

// ✅ After
fn read_config() -> DaemonResult<Config> {
    let path = "/etc/keyrx/config.json";
    let content = fs::read_to_string(path)
        .map_err(|e| DaemonError::Config(ConfigError::FileNotFound {
            path: path.to_string(),
        }))?;

    let config = serde_json::from_str(&content)
        .map_err(|e| DaemonError::Config(ConfigError::Parse(e.to_string())))?;

    Ok(config)
}
```

### Pattern 2: JSON Serialization

```rust
// ❌ Before
fn serialize_profile(profile: &Profile) -> String {
    serde_json::to_string(profile).unwrap()
}

// ✅ After
fn serialize_profile(profile: &Profile) -> DaemonResult<String> {
    Ok(serde_json::to_string(profile)?)
}
```

### Pattern 3: Platform Operations

```rust
// ❌ Before
fn open_input_device(path: &str) -> Device {
    evdev::Device::open(path).unwrap()
}

// ✅ After
fn open_input_device(path: &str) -> Result<Device, PlatformError> {
    evdev::Device::open(path).map_err(|e| {
        PlatformError::DeviceAccess {
            device: path.to_string(),
            reason: e.to_string(),
            suggestion: "Ensure the device exists and you have permission to access it".to_string(),
        }
    })
}
```

### Pattern 4: Option to Result

```rust
// ❌ Before
fn get_active_profile() -> Profile {
    ACTIVE_PROFILE.lock().unwrap().clone().unwrap()
}

// ✅ After
fn get_active_profile() -> DaemonResult<Profile> {
    // SAFETY: Mutex cannot be poisoned
    #[allow(clippy::unwrap_used)]
    let guard = ACTIVE_PROFILE.lock().unwrap();

    guard.clone().ok_or_else(|| {
        DaemonError::Config(ConfigError::InvalidProfile {
            name: "none".to_string(),
            reason: "No active profile set".to_string(),
        })
    })
}
```

## Testing Error Handling

### Unit Tests

Test both success and error paths:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_success() {
        let config = load_config("test/valid_config.json").unwrap();
        assert_eq!(config.version, 1);
    }

    #[test]
    fn test_load_config_file_not_found() {
        let result = load_config("nonexistent.json");
        assert!(result.is_err());

        match result.unwrap_err() {
            DaemonError::Config(ConfigError::FileNotFound { path }) => {
                assert!(path.contains("nonexistent.json"));
            }
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_load_config_invalid_json() {
        let result = load_config("test/invalid.json");
        assert!(result.is_err());

        match result.unwrap_err() {
            DaemonError::Config(ConfigError::Parse(_)) => {}
            _ => panic!("Expected Parse error"),
        }
    }
}
```

### Integration Tests

Test error propagation through full stack:

```rust
#[tokio::test]
async fn test_api_error_response() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/profiles/nonexistent")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = serde_json::from_slice(
        &hyper::body::to_bytes(response.into_body()).await.unwrap()
    ).unwrap();

    assert!(body["error"].as_str().unwrap().contains("not found"));
}
```

## Clippy Enforcement

### Workspace Configuration

Error handling is enforced through clippy lints in `Cargo.toml`:

```toml
[workspace.lints.clippy]
unwrap_used = "deny"   # No unwrap() allowed (use #[allow] for safe cases)
expect_used = "deny"   # No expect() allowed (use #[allow] for safe cases)
```

### Running Clippy

```bash
# Check for unwrap/expect violations
cargo clippy --workspace -- -D warnings

# Fix automatically where possible
cargo clippy --workspace --fix
```

### Adding Allow Attributes

When unwrap is truly safe, add an allow attribute with SAFETY comment:

```rust
// SAFETY: [Explain why this unwrap is safe]
#[allow(clippy::unwrap_used)]
let value = option.unwrap();
```

## Troubleshooting

### "Cannot use `?` in a function that returns `()`"

**Problem:** Function doesn't return a Result type.

**Solution:** Change function signature to return `DaemonResult<()>`:

```rust
// Before
fn process_data() {
    let data = load_data()?; // Error: cannot use `?`
}

// After
fn process_data() -> DaemonResult<()> {
    let data = load_data()?;
    Ok(())
}
```

### "The trait `From<ErrorType>` is not implemented"

**Problem:** Automatic error conversion not set up.

**Solution:** Add `#[from]` attribute or implement conversion:

```rust
// Option 1: Add #[from] to error enum
#[derive(Error, Debug)]
pub enum DaemonError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

// Option 2: Implement From manually
impl From<std::io::Error> for DaemonError {
    fn from(err: std::io::Error) -> Self {
        DaemonError::Io(err)
    }
}

// Option 3: Use map_err
let content = fs::read_to_string(path)
    .map_err(|e| DaemonError::Io(e))?;
```

### "Error messages are not helpful"

**Problem:** Generic error messages don't provide context.

**Solution:** Add context when converting errors:

```rust
// ❌ Not helpful
let config = serde_json::from_str(&content)?;

// ✅ Helpful
let config = serde_json::from_str(&content)
    .map_err(|e| DaemonError::Config(ConfigError::Parse(
        format!("Failed to parse profile '{}': {}", profile_name, e)
    )))?;
```

### "Clippy complains about unwrap in tests"

**Problem:** Clippy denies unwrap even in test code.

**Solution:** Tests are allowed to unwrap (tests can panic):

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_something() {
        let result = some_function().unwrap(); // OK in tests
        assert_eq!(result, expected);
    }
}
```

## Best Practices

1. **Return Result types**: All fallible operations should return `Result<T, E>`

2. **Use specific error types**: Define custom error types with `thiserror` for better type safety

3. **Add error context**: Include relevant information (paths, names, reasons) in error messages

4. **Don't expose internal errors**: Map internal errors to user-friendly messages in public APIs

5. **Log errors with context**: Use structured logging with error details

6. **Test error paths**: Write tests for both success and error scenarios

7. **Document safe unwraps**: Always add SAFETY comments when using `#[allow(clippy::unwrap_used)]`

8. **Propagate errors upward**: Let errors bubble up to the appropriate handling layer

9. **Handle errors gracefully**: Prefer error recovery over crashing when possible

10. **Provide helpful suggestions**: Include actionable suggestions in error messages

## Migration Checklist

When migrating a module to proper error handling:

- [ ] Add Result return types to all fallible functions
- [ ] Replace `unwrap()` with `?` operator
- [ ] Add context to errors with descriptive messages
- [ ] Define custom error types if needed
- [ ] Update function signatures in tests
- [ ] Test both success and error paths
- [ ] Add SAFETY comments for legitimate unwraps
- [ ] Run `cargo clippy` to verify no violations
- [ ] Update documentation to show error types

## References

### Official Documentation
- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [anyhow documentation](https://docs.rs/anyhow/)
- [thiserror documentation](https://docs.rs/thiserror/)
- [Clippy lints reference](https://rust-lang.github.io/rust-clippy/master/)

### keyrx2 Error Types
- `keyrx_core/src/error.rs` - Core library errors
- `keyrx_daemon/src/error.rs` - Daemon error hierarchy
- `keyrx_daemon/src/cli/error.rs` - CLI error formatting (if exists)
- `keyrx_daemon/src/platform/error.rs` - Platform error types (if exists)

### Related Documentation
- `docs/UNWRAP_HANDLING_STRATEGY.md` - Migration strategy and rationale
- `.spec-workflow/specs/error-handling-migration/` - Migration specification
