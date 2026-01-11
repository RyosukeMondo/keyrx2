# keyrx_daemon

OS-level keyboard interception daemon with embedded web server.

## Purpose

`keyrx_daemon` is the runtime component of KeyRx that:
- Intercepts keyboard events at the OS level (Linux via evdev, Windows via low-level hooks)
- Applies remapping rules using the compiled configuration from `keyrx_compiler`
- Provides a web interface for monitoring and configuration
- Serves the React UI built by `keyrx_ui`

## Features

### Platform-Specific Input Handling

- **Linux**: Uses `evdev` for reading input devices and `uinput` for emitting remapped events
- **Windows**: Uses low-level keyboard hooks from `windows-sys`
- Feature-gated to compile only relevant platform code

### Web Server (Default Feature)

- Built with `axum` for REST API and static file serving
- WebSocket support for real-time event streaming
- Serves the compiled UI from `ui_dist/` directory

### Device Management

- **Device Registry**: Persistent storage of device metadata (name, layout)
- **Global Layout Settings**: Default keyboard layout for newly detected devices
- **Rhai-Driven Scope**: Device scope (global vs device-specific) determined by Rhai script, not API

## Build Features

The daemon supports multiple feature flags:

```bash
# Default build (web server only, no platform code)
cargo build --bin keyrx_daemon

# Linux build with input interception
cargo build --bin keyrx_daemon --features linux

# Windows build with input interception
cargo build --bin keyrx_daemon --features windows

# Web-only build (testing/development)
cargo build --bin keyrx_daemon --features web
```

## Usage

```bash
# Run daemon with default settings
cargo run --bin keyrx_daemon

# Run with custom config file
cargo run --bin keyrx_daemon -- --config path/to/config.bin

# Run in headless mode (no browser launch)
cargo run --bin keyrx_daemon -- --headless

# Run with debug logging
cargo run --bin keyrx_daemon -- --log-level debug
```

## Architecture

```
keyrx_daemon/
├── src/
│   ├── main.rs              # Entry point and CLI
│   ├── cli/                 # CLI commands
│   │   ├── mod.rs          # CLI routing
│   │   ├── common.rs       # Shared CLI output formatting
│   │   ├── config.rs       # Configuration management
│   │   ├── device.rs       # Device management commands
│   │   └── profile.rs      # Profile management commands
│   ├── platform/            # Platform-specific input handling
│   │   ├── mod.rs          # Platform abstraction
│   │   ├── linux.rs        # Linux evdev/uinput implementation
│   │   └── windows.rs      # Windows hooks implementation
│   ├── services/            # Business logic services
│   │   ├── device_registry.rs  # Device metadata storage
│   │   └── settings_service.rs # Global settings management
│   └── web/                 # Web server components
│       ├── mod.rs          # Server setup and routing
│       ├── handlers/       # REST API handlers
│       │   ├── devices.rs  # Device endpoints
│       │   ├── profile.rs  # Profile endpoints
│       │   └── settings.rs # Settings endpoints
│       ├── ws.rs           # WebSocket handlers
│       └── static_files.rs # UI file serving
└── ui_dist/                 # Embedded UI files (from keyrx_ui build)
```

## Dependencies

- `keyrx_core`: Core remapping logic
- `axum`: Web framework (optional, enabled by default)
- `tower-http`: HTTP middleware for static files and CORS (optional)
- `tokio`: Async runtime (optional, for web server)
- `evdev`: Linux input device reading (optional, Linux only)
- `uinput`: Linux input device emulation (optional, Linux only)
- `nix`: Unix system calls (optional, Linux only)
- `windows-sys`: Windows API bindings (optional, Windows only)

## Development

The daemon is designed to be run via the automation scripts:

```bash
# Build daemon
./scripts/build.sh

# Launch daemon with default config
./scripts/launch.sh

# Launch in debug mode
./scripts/launch.sh --debug

# Launch with custom config
./scripts/launch.sh --config path/to/config.bin
```

## REST API

### Device Endpoints

#### GET /api/devices

List all detected devices with metadata.

**Response:**
```json
{
  "devices": [
    {
      "serial": "SERIAL_ABC123",
      "name": "My Keyboard",
      "vendor_id": "1234",
      "product_id": "5678",
      "layout": "ANSI_104",
      "connected": true,
      "last_seen": 1704067200000
    }
  ]
}
```

**Fields:**
- `serial`: Unique device identifier
- `name`: User-assigned device name (editable)
- `vendor_id`: USB vendor ID
- `product_id`: USB product ID
- `layout`: Keyboard layout (`ANSI_104`, `ISO_105`, `JIS_109`, `HHKB`, `NUMPAD`)
- `connected`: Whether device is currently connected
- `last_seen`: Unix timestamp (milliseconds) of last detection

#### PATCH /api/devices/:serial

Update device metadata (name and layout).

**Request:**
```json
{
  "name": "Gaming Keyboard",
  "layout": "ISO_105"
}
```

**Response:**
```json
{
  "serial": "SERIAL_ABC123",
  "name": "Gaming Keyboard",
  "layout": "ISO_105",
  "connected": true
}
```

**Validation:**
- `name`: 1-100 characters
- `layout`: Must be valid enum value

**Changes from Previous Version:**
- **Removed**: `scope` field (global/device-specific) - now determined by Rhai script
- Device scope is implicit in the Rhai configuration via `device()` blocks

#### DELETE /api/devices/:serial

Forget a device (remove from registry).

**Response:**
```json
{
  "success": true
}
```

### Settings Endpoints

#### GET /api/settings/global-layout

Get the global default keyboard layout.

**Response:**
```json
{
  "layout": "ANSI_104"
}
```

**Default**: If not set, returns `null`

#### PUT /api/settings/global-layout

Set the global default keyboard layout for newly detected devices.

**Request:**
```json
{
  "layout": "ISO_105"
}
```

**Response:**
```json
{
  "layout": "ISO_105"
}
```

**Validation:**
- `layout`: Must be `ANSI_104`, `ISO_105`, `JIS_109`, `HHKB`, or `NUMPAD`
- Returns HTTP 400 for invalid layout values

**Persistence**: Saved to `settings.json` in daemon config directory

**Device Layout Inheritance:**
1. **New devices**: Inherit global layout by default
2. **Device-specific override**: Setting device layout via PATCH overrides global
3. **Global change**: Does not affect existing device overrides

### Profile Endpoints

#### GET /api/profiles

List all available profiles.

**Response:**
```json
{
  "profiles": [
    {
      "name": "default",
      "path": "/home/user/.config/keyrx/profiles/default.rhai",
      "active": true,
      "created": 1704067200000,
      "modified": 1704070800000
    }
  ]
}
```

#### POST /api/profiles

Create a new profile.

**Request:**
```json
{
  "name": "gaming",
  "template": "blank"
}
```

**Templates:**
- `blank`: Empty Rhai script with comments
- `default`: Basic passthrough configuration
- `vim`: Vim-style navigation layer

**Response:**
```json
{
  "name": "gaming",
  "path": "/home/user/.config/keyrx/profiles/gaming.rhai"
}
```

#### GET /api/profiles/:name/config

Get the Rhai script content for a profile.

**Response:**
```json
{
  "name": "gaming",
  "content": "// Rhai script content\nmap(Key::A, Key::B);\n"
}
```

#### PUT /api/profiles/:name/config

Update the Rhai script content for a profile.

**Request:**
```json
{
  "content": "// Updated Rhai script\nmap(Key::CapsLock, Key::Escape);\n"
}
```

**Response:**
```json
{
  "success": true
}
```

**Validation**: Rhai script is validated before saving (syntax errors return HTTP 400)

### WebSocket RPC

Connect to `ws://localhost:3030/ws` for real-time updates.

**RPC Request Format:**
```json
{
  "id": "uuid-v4",
  "method": "get_devices",
  "params": {}
}
```

**RPC Response Format:**
```json
{
  "id": "uuid-v4",
  "result": { "devices": [...] },
  "error": null
}
```

**Available Methods:**
- `get_devices`: List all devices
- `set_device_name`: Update device name
- `set_device_layout`: Update device layout
- `get_profiles`: List all profiles
- `activate_profile`: Activate a profile
- `get_latency_stats`: Get keystroke latency statistics

**Real-time Events:**
- `device_connected`: Device connected
- `device_disconnected`: Device disconnected
- `profile_activated`: Profile changed
- `latency_update`: Latency statistics update

## Device Registry

### Storage Format

Device metadata is stored in `device_registry.json` in the daemon config directory.

**Format:**
```json
{
  "devices": {
    "SERIAL_ABC123": {
      "name": "My Keyboard",
      "layout": "ANSI_104",
      "last_seen": 1704067200000
    }
  }
}
```

### Migration from Old Format

**Backward Compatibility**: The daemon automatically handles old registry files that include the deprecated `scope` field.

**Old Format (deprecated):**
```json
{
  "devices": {
    "SERIAL_ABC123": {
      "name": "My Keyboard",
      "layout": "ANSI_104",
      "scope": "Global",  // ← Deprecated field
      "last_seen": 1704067200000
    }
  }
}
```

**Migration Behavior:**
1. Old registry files load successfully
2. `scope` field is ignored during deserialization
3. On next save, `scope` field is removed
4. No manual migration required

**Rationale**: Device scope is now determined by the Rhai script itself via `device()` blocks, not by daemon metadata. This ensures the Rhai script is the Single Source of Truth (SSOT) for all configuration.

### Global Settings Storage

Global settings (like default layout) are stored in `settings.json`.

**Format:**
```json
{
  "global_layout": "ANSI_104"
}
```

**Atomic Writes**: Settings are written atomically (write to temp file, then rename) to prevent corruption.

## Architectural Changes

### Rhai-Driven Scope

**Previous Architecture**: Device scope (global vs device-specific) was a user-configurable field in the UI and stored in the device registry.

**New Architecture**: Device scope is determined solely by the Rhai script:

- **Global Mappings**: Mappings outside `device()` blocks apply to all devices
- **Device-Specific Mappings**: Mappings inside `device("SERIAL")` blocks apply only to that device

**Example Rhai Script:**
```rhai
// Global mapping (applies to all devices)
map(Key::CapsLock, Key::Escape);

// Device-specific mapping
device("SERIAL_ABC123") {
    map(Key::A, Key::B);
}
```

**Benefits:**
- **Single Source of Truth**: Rhai script is authoritative
- **No Configuration Drift**: UI cannot show different scope than Rhai defines
- **Deterministic Behavior**: Visual editor reflects exactly what Rhai script specifies

### API Changes Summary

**Removed Endpoints:**
- `PATCH /api/devices/:serial` no longer accepts `scope` field

**New Endpoints:**
- `GET /api/settings/global-layout`
- `PUT /api/settings/global-layout`

**Changed Structures:**
- `DeviceEntry`: Removed `scope` field
- `DeviceRpcInfo`: Removed `scope` field

**Removed CLI Commands:**
- `keyrx_daemon device set-scope` (scope now determined by Rhai)

## Testing

### Unit Tests

Unit tests are included for each module:

```bash
cargo test --bin keyrx_daemon
```

### Integration Tests

Integration tests verify platform-specific functionality in `tests/`.

**API Tests:**
- `tests/api_devices_test.rs` - Device API endpoints (16 tests)
- `tests/api_settings_test.rs` - Global layout settings API (12 tests)
- `tests/api_contracts_test.rs` - API contract validation (25 tests)

**Test Coverage:**
- Scope removal verification (registry does not contain scope, PATCH ignores scope)
- Global layout API (set, get, persist, validate, atomic writes)
- Device layout inheritance (new devices inherit global, device overrides, precedence)
- Backward compatibility with old registry format (scope field ignored)
- Request validation (invalid layouts return HTTP 400)
- Persistence verification (settings survive daemon restart)

**Run API tests:**
```bash
cargo test api_devices_test
cargo test api_settings_test
cargo test api_contracts_test
```

### Test Infrastructure

Tests use `TestApp` fixture for isolated testing:
- Each test gets isolated config directory
- Automatic cleanup after tests
- Parallel test execution safe
- Mock WebSocket connections

## Type Generation

This crate uses [typeshare](https://github.com/1Password/typeshare) to automatically generate TypeScript type definitions from Rust structs for the frontend.

### Generating TypeScript Types

To regenerate TypeScript types after modifying API structs:

```bash
cd keyrx_daemon
typeshare --lang=typescript --output-file=../keyrx_ui/src/types/generated.ts src/
```

This will scan all Rust source files in `src/` for structs and enums annotated with `#[typeshare]` and generate corresponding TypeScript definitions in `keyrx_ui/src/types/generated.ts`.

### Adding New API Types

When adding new API request/response types:

1. Add the `#[typeshare]` attribute to the struct/enum:
   ```rust
   use typeshare::typeshare;

   #[typeshare]
   #[derive(Serialize, Deserialize)]
   pub struct MyApiResponse {
       pub field: String,
   }
   ```

2. For `u64` and `usize` fields, add serialization hints:
   ```rust
   #[typeshare]
   #[derive(Serialize, Deserialize)]
   pub struct MyStats {
       #[typeshare(serialized_as = "number")]
       pub timestamp: u64,
       #[typeshare(serialized_as = "number")]
       pub count: usize,
   }
   ```

3. Regenerate types:
   ```bash
   cd keyrx_daemon
   typeshare --lang=typescript --output-file=../keyrx_ui/src/types/generated.ts src/
   ```

4. Verify TypeScript compilation:
   ```bash
   cd ../keyrx_ui
   npm run type-check
   ```

### Configuration

Type generation is configured in `Cargo.toml`:

```toml
[package.metadata.typeshare]
output_directory = "../keyrx_ui/src/types"
```

This ensures consistency between Rust and TypeScript types, preventing API contract drift.

## Security Considerations

The daemon requires elevated privileges to intercept keyboard events:
- **Linux**: Requires access to `/dev/input/eventX` and `/dev/uinput` (typically via udev rules or root)
- **Windows**: Requires administrator privileges for low-level keyboard hooks

Always verify the integrity of configuration files before loading them into the daemon.

## License

See the workspace LICENSE file for licensing information.
