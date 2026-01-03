# Requirements Document - Error Handling Migration

## Introduction

This specification defines the migration from ad-hoc `unwrap()` and `expect()` calls to industry-standard error handling using `anyhow` and `thiserror` libraries. The migration will improve code reliability, provide better error messages with context, and establish maintainable error handling patterns across the keyrx2 workspace.

**Value to users:**
- More informative error messages when operations fail
- Reduced unexpected panics in production
- Better debugging experience with error context chains
- Professional error handling aligned with Rust ecosystem standards

## Alignment with Product Vision

This feature supports the keyrx2 product vision by:
- **Reliability**: Replacing panic-prone unwraps with graceful error handling
- **Maintainability**: Establishing consistent error patterns across all crates
- **Developer Experience**: Providing clear error messages that help diagnose issues quickly
- **Code Quality**: Upgrading from manual unwrap counting (fragile) to compiler-enforced quality gates (robust)

## Requirements

### Requirement 1: Add Error Handling Dependencies

**User Story:** As a developer, I want standardized error handling libraries integrated into the workspace, so that I can write robust error handling code following Rust best practices.

#### Acceptance Criteria

1. WHEN workspace dependencies are configured THEN `anyhow` SHALL be added for application code (keyrx_daemon)
2. WHEN workspace dependencies are configured THEN `thiserror` SHALL be added for library code (keyrx_core, keyrx_compiler)
3. WHEN dependencies are added THEN existing code SHALL continue to compile without breaking changes
4. WHEN `cargo build` is executed THEN all crates SHALL successfully compile with new dependencies

### Requirement 2: Define Custom Error Types

**User Story:** As a developer, I want well-defined custom error types for each module, so that I can handle errors appropriately and provide meaningful error messages to users.

#### Acceptance Criteria

1. WHEN defining error types THEN each major module SHALL have a dedicated error enum using `thiserror`
2. WHEN errors occur THEN error types SHALL include context about what operation failed and why
3. WHEN errors are displayed THEN error messages SHALL be user-friendly and actionable
4. IF an error wraps a lower-level error THEN the original error SHALL be preserved using `#[from]` attribute
5. WHEN errors are created THEN they SHALL implement `std::error::Error` trait automatically via `thiserror`

### Requirement 3: Migrate CLI Module Error Handling

**User Story:** As a CLI user, I want clear error messages when commands fail, so that I can understand what went wrong and how to fix it.

#### Acceptance Criteria

1. WHEN CLI commands fail THEN errors SHALL provide context about which operation failed
2. WHEN file operations fail THEN error messages SHALL include the file path that caused the failure
3. WHEN parsing fails THEN error messages SHALL indicate what input was invalid
4. IF an error is recoverable THEN the system SHALL provide suggestions for resolution
5. WHEN errors are displayed THEN they SHALL use `anyhow::Context` to add operation context

### Requirement 4: Migrate Configuration Module Error Handling

**User Story:** As a developer, I want configuration loading failures to provide clear diagnostics, so that I can quickly identify and fix configuration issues.

#### Acceptance Criteria

1. WHEN configuration files are missing THEN errors SHALL indicate the expected file path
2. WHEN configuration parsing fails THEN errors SHALL show the line number and parsing error
3. WHEN configuration validation fails THEN errors SHALL list all validation failures, not just the first one
4. IF configuration is invalid THEN error messages SHALL explain what is wrong and how to fix it
5. WHEN profile compilation fails THEN errors SHALL include the profile name and compilation stage

### Requirement 5: Migrate Platform Module Error Handling

**User Story:** As a developer, I want platform-specific operations to handle errors gracefully, so that the system degrades gracefully rather than panicking.

#### Acceptance Criteria

1. WHEN device access fails THEN errors SHALL indicate which device and what permission is needed
2. WHEN keyboard injection fails THEN errors SHALL provide recovery options
3. WHEN WebSocket connections fail THEN errors SHALL include connection details for debugging
4. IF a platform operation is unsupported THEN errors SHALL clearly state the limitation
5. WHEN errors occur in event loops THEN they SHALL be logged but not crash the daemon

### Requirement 6: Update Quality Gates

**User Story:** As a developer, I want automated quality gates to prevent unwrap regressions, so that error handling quality is maintained long-term.

#### Acceptance Criteria

1. WHEN clippy lints are configured THEN `unwrap_used` SHALL be set to `warn` initially, then upgraded to `deny`
2. WHEN new unwraps are added THEN clippy SHALL generate warnings during development
3. WHEN code is committed THEN pre-commit hooks SHALL enforce clippy lint compliance
4. IF legitimate unwraps exist THEN they SHALL have `#[allow(clippy::unwrap_used)]` with SAFETY comments
5. WHEN manual unwrap counting is replaced THEN the `scripts/check_unwraps.sh` SHALL be deprecated

### Requirement 7: Preserve Backward Compatibility

**User Story:** As a developer, I want the error handling migration to be non-breaking, so that existing code continues to work during the transition.

#### Acceptance Criteria

1. WHEN migrating modules THEN public APIs SHALL maintain the same signatures initially
2. WHEN adding new error types THEN existing error types SHALL continue to work until fully migrated
3. WHEN refactoring error handling THEN tests SHALL continue to pass
4. IF breaking changes are necessary THEN they SHALL be documented in migration notes
5. WHEN migration is complete THEN all tests SHALL pass with improved error handling

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Each error type should represent errors from a single module or domain
- **Modular Design**: Error handling utilities should be isolated and reusable across crates
- **Dependency Management**: Minimize error type dependencies between modules
- **Clear Interfaces**: Define clean error type hierarchies with explicit conversion paths

### Performance
- Error handling SHALL NOT introduce measurable performance overhead in hot paths
- Error context building SHALL use zero-cost abstractions where possible
- Error types SHALL be lightweight and cheap to construct
- Error propagation SHALL use `?` operator for minimal overhead

### Security
- Error messages SHALL NOT leak sensitive information (passwords, tokens, internal paths)
- Error context SHALL sanitize file paths to avoid exposing system internals
- Stack traces SHALL be available in debug builds but omitted in release builds
- Error logs SHALL follow structured logging patterns to prevent injection attacks

### Reliability
- Error handling SHALL never panic in production code except for programmer errors
- Errors SHALL be propagated correctly through all layers
- Error recovery SHALL be attempted where appropriate
- System SHALL degrade gracefully when errors occur in non-critical paths

### Usability
- Error messages SHALL be written for end users, not just developers
- Error context SHALL include actionable information for resolution
- Error formatting SHALL be consistent across all modules
- CLI error output SHALL be colored and formatted for readability

### Maintainability
- Error types SHALL be defined close to the code they represent
- Error handling patterns SHALL be documented with examples
- Migration guide SHALL be provided for developers
- Legacy unwraps SHALL be documented with migration tracking comments
