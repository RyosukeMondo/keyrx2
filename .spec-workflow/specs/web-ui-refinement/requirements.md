# Requirements Document

## Introduction

The web UI needs refinement to align with the Rhai-driven architecture where device scope (global vs device-specific) is determined by the Rhai script itself, not by user toggles. The current implementation treats scope as a user-configurable setting, creating a disconnect between the visual editor and the underlying Rhai configuration. This refinement will establish a clear, intuitive workflow where:

1. **Devices Page**: Manages device metadata (name, layout) without scope configuration
2. **Profile Page**: Lists profiles with Rhai file paths and auto-generates defaults
3. **Config Page**: Provides bidirectional sync between visual editor and Rhai script with device-aware configuration

This aligns with the product vision of "AI Coding Agent First" by making the Rhai script the single source of truth (SSOT) for all configuration, including scope.

## Alignment with Product Vision

This feature supports the following goals from product.md:

- **Single Source of Truth (SSOT)**: Rhai script is the authoritative source for all configuration, including device scope
- **AI Coding Agent First**: Eliminates UI-level scope toggles that could create drift from Rhai script
- **Deterministic Behavior**: Visual editor reflects exactly what the Rhai script defines
- **Zero Configuration Drift**: Frontend cannot show different behavior than what Rhai script specifies

## Requirements

### Requirement 1: Devices Page - Remove Scope Selection

**User Story:** As a keyboard remapping user, I want the Devices page to only manage device metadata (name, layout) without scope configuration, so that scope is determined solely by my Rhai script.

#### Acceptance Criteria

1. WHEN I open the Devices page THEN the system SHALL display all detected devices with name, serial, vendor ID, product ID, and last seen timestamp
2. WHEN I view a device card THEN the system SHALL NOT display any scope selector (global/device-specific toggle)
3. WHEN I edit a device THEN the system SHALL allow me to rename the device and select keyboard layout ONLY
4. WHEN I save device changes THEN the system SHALL persist name and layout but NOT scope
5. IF a device has a layout assigned THEN the system SHALL display the layout name (ANSI 104, ISO 105, JIS 109, HHKB, NUMPAD)

### Requirement 2: Devices Page - Add Global Layout Selection

**User Story:** As a keyboard remapping user, I want to select a default keyboard layout globally, so that new devices inherit this layout.

#### Acceptance Criteria

1. WHEN I open the Devices page THEN the system SHALL display a "Global Settings" card at the top
2. WHEN I view the Global Settings card THEN the system SHALL show a layout selector with options: ANSI 104, ISO 105, JIS 109, HHKB, NUMPAD
3. WHEN I change the global layout THEN the system SHALL save it to daemon configuration
4. WHEN a new device is detected THEN the system SHALL assign the global layout by default
5. WHEN I change a device's layout THEN the system SHALL override the global layout for that specific device

### Requirement 3: Profile Page - Display Rhai File Path

**User Story:** As a keyboard remapping user, I want to see the Rhai file path for each profile, so that I understand which script file the profile uses.

#### Acceptance Criteria

1. WHEN I view a profile card THEN the system SHALL display the Rhai file path prominently (e.g., "~/.config/keyrx/profiles/gaming.rhai")
2. WHEN I hover over the file path THEN the system SHALL show the full absolute path as a tooltip
3. WHEN I click the file path THEN the system SHALL navigate to the Config page for that profile
4. IF the Rhai file does not exist THEN the system SHALL display an error badge on the profile card

### Requirement 4: Profile Page - Auto-Generate Default Profile

**User Story:** As a first-time user, I want a default profile to be created automatically if none exist, so that I can start using the system immediately.

#### Acceptance Criteria

1. WHEN I open the Profile page for the first time AND no profiles exist THEN the system SHALL automatically create a default profile named "default"
2. WHEN the default profile is created THEN the system SHALL use the "blank" template
3. WHEN the default profile is created THEN the system SHALL display a notification: "Default profile created. Click 'Edit' to customize."
4. WHEN the default profile exists THEN the system SHALL activate it automatically
5. IF profile creation fails THEN the system SHALL display an error with actionable guidance

### Requirement 5: Config Page - Device-Aware Configuration

**User Story:** As a keyboard remapping user, I want to select specific devices to configure (not just global vs device-specific toggle), so that I can create device-specific mappings.

#### Acceptance Criteria

1. WHEN I open the Config page THEN the system SHALL display a "Device Selector" with checkboxes for: Global, Device A, Device B, Device C, etc.
2. WHEN I check "Global" THEN the system SHALL show the global key layout
3. WHEN I check one or more specific devices THEN the system SHALL show key layouts for those devices side-by-side
4. WHEN I have "Global" checked AND a specific device checked THEN the system SHALL show both layouts with clear labels
5. WHEN I uncheck all devices THEN the system SHALL display a warning: "Select at least one device or global to configure"
6. WHEN I save configuration THEN the system SHALL generate Rhai script with appropriate device-specific blocks

### Requirement 6: Config Page - Bidirectional Rhai Sync

**User Story:** As a keyboard remapping user, I want changes in the visual editor to update the Rhai script and vice versa, so that both views stay in sync.

#### Acceptance Criteria

1. WHEN I open a profile in Config page THEN the system SHALL parse the Rhai script and reflect all mappings in the visual editor
2. WHEN I change a key mapping in the visual editor THEN the system SHALL update the Rhai script immediately
3. WHEN I edit the Rhai script directly THEN the system SHALL parse it and update the visual editor within 500ms
4. IF the Rhai script has syntax errors THEN the system SHALL display the error in the code editor and disable visual editor updates
5. IF the Rhai script has device-specific blocks THEN the system SHALL reflect them in the device selector
6. WHEN I switch between visual and code editor tabs THEN the system SHALL maintain sync state
7. IF parsing fails THEN the system SHALL show the last valid state in visual editor with a warning banner

### Requirement 7: Config Page - Rhai Parser Integration

**User Story:** As a keyboard remapping user, I want the system to parse my Rhai script accurately, so that device-specific configurations are correctly displayed.

#### Acceptance Criteria

1. WHEN the system parses a Rhai script THEN it SHALL identify all device() blocks with serial numbers
2. WHEN the system parses a Rhai script THEN it SHALL extract all key mappings (simple, tap_hold, macro, layer_switch)
3. WHEN the system parses a Rhai script THEN it SHALL identify which mappings belong to global scope vs device-specific scope
4. WHEN the system detects device() blocks THEN it SHALL populate the device selector with those devices
5. IF a device in the Rhai script is not currently connected THEN the system SHALL still show it in the device selector with a "disconnected" badge
6. WHEN parsing fails THEN the system SHALL provide line number and error message from the Rhai engine

### Requirement 8: Config Page - Rhai Code Generation

**User Story:** As a keyboard remapping user, I want visual editor changes to generate clean, readable Rhai code, so that I can understand and manually edit it later.

#### Acceptance Criteria

1. WHEN I create a simple key mapping in visual editor THEN the system SHALL generate: `map(Key::A, Key::B);`
2. WHEN I create a tap-hold mapping THEN the system SHALL generate: `map(Key::CapsLock, tap_hold(Key::Escape, Key::LCtrl, 200));`
3. WHEN I create a device-specific mapping THEN the system SHALL generate a device() block with the correct serial number
4. WHEN I create multiple device-specific mappings for the same device THEN the system SHALL group them in a single device() block
5. WHEN I delete a key mapping THEN the system SHALL remove the corresponding Rhai line
6. WHEN the generated Rhai code is formatted THEN it SHALL follow project formatting standards (4-space indentation, consistent spacing)
7. IF I manually edit Rhai and then make visual changes THEN the system SHALL preserve comments and formatting where possible

### Requirement 9: Rust-TypeScript Interface Validation

**User Story:** As a developer, I want TypeScript types to automatically match Rust structs, so that interface changes are caught at compile-time and runtime errors are prevented.

#### Acceptance Criteria

1. WHEN Rust API structs are modified THEN TypeScript types SHALL be automatically regenerated to match
2. WHEN daemon sends API responses THEN the system SHALL validate response structure matches TypeScript types
3. WHEN UI sends API requests THEN the system SHALL validate request structure matches Rust expectations
4. IF TypeScript types are out of sync with Rust structs THEN the build SHALL fail with clear error messages
5. WHEN WebSocket RPC messages are sent THEN the system SHALL validate message structure at runtime
6. IF an API response has unexpected fields THEN the system SHALL log a warning and include the extra fields in error details
7. WHEN running CI/CD THEN the system SHALL verify TypeScript types match Rust structs
8. IF a breaking API change is detected THEN the system SHALL fail the build with diff showing what changed

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility Principle**: Each component handles one concern (DeviceSelector, RhaiParser, RhaiCodeGen)
- **Modular Design**: Rhai parsing and code generation in separate utility modules (`utils/rhaiParser.ts`, `utils/rhaiCodeGen.ts`)
- **Dependency Management**: Parser depends on Rhai grammar specification, not on UI components
- **Clear Interfaces**: TypeScript interfaces for parsed Rhai AST, device blocks, and key mappings

### Performance

- **Parsing Latency**: Rhai parsing SHALL complete within 100ms for scripts up to 10,000 lines
- **Visual Editor Update**: Visual editor SHALL reflect Rhai changes within 500ms
- **Code Generation**: Rhai code generation SHALL complete within 50ms for up to 1,000 mappings
- **Debouncing**: Code editor changes SHALL be debounced (500ms) before triggering parse

### Security

- **Script Validation**: All Rhai scripts SHALL be validated by daemon before compilation
- **No Arbitrary Execution**: Frontend SHALL NOT execute Rhai scripts (parsing only)
- **Sanitization**: Device serial numbers SHALL be sanitized before insertion into Rhai code
- **API Validation**: All API requests/responses SHALL be validated against schemas to prevent injection attacks
- **Type Safety**: TypeScript types SHALL be derived from Rust structs to prevent type confusion vulnerabilities

### Reliability

- **Graceful Degradation**: If Rhai parsing fails, visual editor SHALL show last valid state
- **Error Recovery**: Parse errors SHALL NOT crash the UI
- **State Persistence**: Unsaved changes SHALL be preserved in browser localStorage

### Usability

- **WCAG 2.2 Level AA**: All UI changes SHALL maintain accessibility compliance
- **Keyboard Navigation**: All new components SHALL support keyboard-only operation
- **Loading States**: Parsing/code generation SHALL display loading indicators
- **Error Messages**: Parse errors SHALL include line number, error type, and suggested fix
- **Undo/Redo**: Code editor SHALL support undo/redo (Ctrl+Z/Ctrl+Shift+Z)
