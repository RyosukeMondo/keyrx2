# Daemon Domain Implementation

**Status**: ✅ Complete

## Overview

This module implements Domain-Driven Design (DDD) patterns for the daemon operations in keyrx. It follows the exact structure and patterns established in the Core domain (`keyrx_core/src/domain/`).

## Structure

| File | Lines | Description |
|------|-------|-------------|
| `mod.rs` | 96 | Domain module root with DomainError enum |
| `aggregates.rs` | 539 | 3 aggregates (Device, Profile, Session) |
| `entities.rs` | 402 | 3 entities (InputDevice, OutputDevice, WebSocketConnection) |
| `value_objects.rs` | 246 | 3 value objects (DeviceSerial, ProfileName, Port) |
| `repositories.rs` | 393 | 3 repository traits (Device, Profile, Settings) |
| `services.rs` | 530 | 3 domain services (DeviceIdentification, ProfileSwitching, WebSocketBroadcast) |
| `events.rs` | 340 | 13 domain events + event bus |
| **Total** | **2546** | **All tests passing (98 tests)** |

## Aggregates

### DeviceAggregate
- **Purpose**: Device lifecycle and state management
- **Root Entity**: InputDeviceEntity
- **Responsibilities**:
  - Attach/detach output devices
  - Activate/deactivate profiles
  - Enable/disable device
  - Track device state and events
- **Invariants**:
  - Only one output device per input device
  - Profile activation requires active device
  - Version counter for optimistic locking

### ProfileAggregate
- **Purpose**: Profile configuration and lifecycle management
- **Root**: ProfileNameVO
- **Responsibilities**:
  - Activate/deactivate profile
  - Attach/detach devices
  - Validate configuration path (.krx file)
- **Invariants**:
  - Profile can only be activated once
  - Config path must end with .krx
  - Device serial must be unique per profile

### SessionAggregate
- **Purpose**: User session state and lifecycle
- **Root**: Session ID (String)
- **Responsibilities**:
  - Add/remove profiles
  - Add/remove devices
  - Track session activity
- **Invariants**:
  - No duplicate profiles in session
  - No duplicate devices in session
  - Activity timestamp always increasing

## Entities

### InputDeviceEntity
- **Identity**: u64 ID
- **Attributes**: Serial, name, state, timestamps
- **Lifecycle**: Active → Inactive → Disconnected
- **Behaviors**: Enable, disable, disconnect, mark active

### OutputDeviceEntity
- **Identity**: u64 ID
- **Attributes**: Name, enabled flag, event count
- **Lifecycle**: Created → Enabled/Disabled
- **Behaviors**: Enable, disable, increment event count

### WebSocketConnectionEntity
- **Identity**: u64 ID
- **Attributes**: Client ID, timestamps, message count, authenticated flag
- **Lifecycle**: Connected → Authenticated → Disconnected
- **Behaviors**: Authenticate, subscribe, record message

## Value Objects

### DeviceSerialVO
- **Validation**: Alphanumeric + hyphens/underscores, 1-128 chars
- **Immutable**: Yes
- **Hash/Eq**: Yes

### ProfileNameVO
- **Validation**: Alphanumeric + spaces/hyphens/underscores, 1-64 chars
- **Immutable**: Yes
- **Hash/Eq**: Yes

### PortVO
- **Validation**: 1024-65535 (unprivileged ports)
- **Immutable**: Yes
- **Hash/Eq**: Yes

## Repository Traits

### DeviceRepository
- `save(&mut self, device: &DeviceAggregate)`
- `load(&self, serial: &DeviceSerialVO)`
- `list(&self)` → Vec<DeviceAggregate>
- `exists(&self, serial: &DeviceSerialVO)`
- `delete(&mut self, serial: &DeviceSerialVO)`
- `list_active(&self)` → Vec<DeviceAggregate>

### ProfileRepository
- `save(&mut self, profile: &ProfileAggregate)`
- `load(&self, name: &ProfileNameVO)`
- `list(&self)` → Vec<ProfileAggregate>
- `exists(&self, name: &ProfileNameVO)`
- `delete(&mut self, name: &ProfileNameVO)`
- `get_active(&self)` → Option<&ProfileAggregate>
- `list_by_device(&self, serial: &DeviceSerialVO)`

### SettingsRepository
- `get(&self, key: &str)` → Option<String>
- `set(&mut self, key: &str, value: String)`
- `delete(&mut self, key: &str)`
- `list_keys(&self)` → Vec<String>
- `clear(&mut self)`
- `exists(&self, key: &str)`

## Domain Services

### DeviceIdentificationService
- **Purpose**: Match physical devices by characteristics
- **Methods**:
  - `identify_by_serial()` - Exact serial match
  - `match_by_name_pattern()` - Fuzzy name matching
  - `validate_device()` - Device validation
- **Event Bus**: Publishes device-related events

### ProfileSwitchingService
- **Purpose**: Manage profile activation/deactivation
- **Methods**:
  - `switch_profile()` - Switch between profiles
  - `attach_profile_to_device()` - Attach profile to device
  - `detach_profile_from_device()` - Detach profile from device
  - `validate_profile()` - Profile validation
- **Event Bus**: Publishes profile-related events

### WebSocketBroadcastService
- **Purpose**: Manage WebSocket connections and broadcasting
- **Methods**:
  - `add_connection()` - Register new connection
  - `remove_connection()` - Unregister connection
  - `authenticate_connection()` - Mark connection as authenticated
  - `subscribe_to_profile()` - Subscribe connection to profile events
  - `broadcast()` - Broadcast to all authenticated connections
  - `broadcast_to_profile()` - Broadcast to profile subscribers
- **Event Bus**: Publishes WebSocket-related events

## Domain Events

| Event | Description |
|-------|-------------|
| `DeviceConnected` | Physical device connected |
| `DeviceDisconnected` | Physical device disconnected |
| `DeviceEnabled` | Device enabled for input |
| `DeviceDisabled` | Device disabled |
| `ProfileActivated` | Profile activated |
| `ProfileDeactivated` | Profile deactivated |
| `ProfileAttachedToDevice` | Profile attached to device |
| `ProfileDetachedFromDevice` | Profile detached from device |
| `WebSocketClientConnected` | WS client connected |
| `WebSocketClientDisconnected` | WS client disconnected |
| `WebSocketClientAuthenticated` | WS client authenticated |
| `SessionStarted` | User session started |
| `SessionEnded` | User session ended |

## Testing

All domain components have comprehensive unit tests:
- ✅ 98 tests passing
- ✅ 0 failures
- ✅ Full coverage of aggregates, entities, value objects, repositories, and services
- ✅ Mock implementations for repository traits
- ✅ Event bus testing

## Design Patterns

### SOLID Principles
- **Single Responsibility**: Each aggregate has one clear purpose
- **Open/Closed**: Extend via traits, not modification
- **Dependency Inversion**: Repositories are traits

### DDD Patterns
- **Aggregates**: Root entities with consistency boundaries
- **Entities**: Objects with unique identity
- **Value Objects**: Immutable objects defined by attributes
- **Repositories**: Persistence abstraction
- **Domain Services**: Business logic that doesn't fit entities
- **Domain Events**: Record what happened

## Integration

To use this domain in the infrastructure layer:

```rust
use keyrx_daemon::domain::{
    DeviceAggregate, ProfileAggregate, SessionAggregate,
    DeviceRepository, ProfileRepository, SettingsRepository,
    DeviceIdentificationService, ProfileSwitchingService,
};

// Implement repository traits
struct FileSystemDeviceRepository { /* ... */ }
impl DeviceRepository for FileSystemDeviceRepository { /* ... */ }

// Use domain services
let mut service = DeviceIdentificationService::new();
let device = service.identify_by_serial(&serial, &devices)?;
```

## Future Work

This domain implementation is ready for use by the infrastructure layer. Next steps:
1. Implement concrete repository implementations (filesystem, database)
2. Wire up domain services to the daemon event loop
3. Integrate event bus with WebSocket broadcasting
4. Add domain event handlers for persistence

## References

- Core domain: `keyrx_core/src/domain/`
- DDD Task: `.spec-workflow/specs/ai-dev-foundation/03-ddd/tasks.md`
- Architecture: `.spec-workflow/steering/structure.md`
