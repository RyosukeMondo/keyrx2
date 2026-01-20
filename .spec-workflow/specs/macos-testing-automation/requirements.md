# Requirements Document

## Introduction

This specification defines automated testing infrastructure for macOS platform support in keyrx. The testing strategy addresses the unique constraint of macOS Accessibility permissions, which prevents E2E tests from running in CI environments. The solution implements a three-layer testing approach:

1. **Layer 1**: Enhanced mock-based tests (no permissions needed, runs in CI)
2. **Layer 2**: E2E tests with permission checks (developer-only, auto-skips in CI)
3. **Layer 3**: Automated test runner script (orchestrates full test suite)

This ensures maximal automated coverage while maintaining a smooth developer experience and CI reliability.

## Alignment with Product Vision

This feature directly supports keyrx's "AI Coding Agent First" philosophy by:
- **Eliminating manual testing barriers**: Automated tests ensure macOS works reliably without requiring human UAT
- **Enabling deterministic verification**: Mock tests provide 100% reproducible results
- **Supporting cross-platform parity**: Ensures macOS achieves the same test coverage as Linux/Windows
- **Reducing deployment risk**: Comprehensive test suite catches regressions before release

The implementation also upholds keyrx's performance standards by:
- Testing sub-millisecond latency requirements with benchmarks
- Validating zero-copy deserialization paths
- Ensuring deterministic state transitions

## Requirements

### Requirement 1: Enhanced Mock Tests (Layer 1)

**User Story:** As a CI pipeline, I want comprehensive mock-based tests that validate macOS-specific logic without requiring Accessibility permissions, so that every commit can be automatically verified.

#### Acceptance Criteria

1. WHEN the test suite runs in CI THEN all mock tests SHALL execute without Accessibility permissions
2. WHEN testing CGEvent conversion THEN tests SHALL validate round-trip conversion (CGEvent → KeyCode → CGEvent) with zero data loss
3. WHEN testing platform initialization THEN tests SHALL verify graceful failure when permissions are denied
4. WHEN testing device discovery THEN tests SHALL validate IOKit device enumeration with mock responses
5. WHEN testing edge cases THEN tests SHALL cover:
   - Multiple USB keyboards simultaneously
   - Bluetooth keyboard disconnection/reconnection
   - Invalid device serial numbers
   - CGKeyCode values outside normal ranges (0-127)
6. WHEN measuring coverage THEN mock tests SHALL achieve ≥90% line coverage of platform/macos modules

### Requirement 2: E2E Tests with Permission Checks (Layer 2)

**User Story:** As a developer with Accessibility permissions enabled, I want E2E tests that spawn real daemon processes and verify hardware-level functionality, so that I can validate end-to-end behavior before release.

#### Acceptance Criteria

1. WHEN running E2E tests in CI THEN tests SHALL automatically skip with informative message (not fail)
2. WHEN running E2E tests locally without permissions THEN tests SHALL print guidance on enabling Accessibility permissions
3. WHEN Accessibility permission is granted THEN E2E tests SHALL:
   - Spawn actual keyrx_daemon process
   - Test real CGEvent capture and injection
   - Validate device discrimination with physical hardware
   - Test timing-sensitive features (tap-hold, combos)
4. WHEN testing basic remapping THEN E2E tests SHALL verify A → B remapping works with real keyboard
5. WHEN testing multi-device discrimination THEN E2E tests SHALL verify device-specific mappings apply correctly
6. WHEN testing tap-hold behavior THEN E2E tests SHALL verify timing accuracy matches QMK-style permissive hold
7. WHEN E2E tests complete THEN daemon process SHALL be cleanly terminated (SIGTERM, then SIGKILL after 5s timeout)

### Requirement 3: Automated Test Runner Script (Layer 3)

**User Story:** As a developer, I want a single script that runs the complete macOS test suite with clear progress reporting, so that I can validate my changes before pushing to CI.

#### Acceptance Criteria

1. WHEN executing test runner script THEN it SHALL run in the following order:
   - Mock-based integration tests (always run)
   - Permission check (non-blocking)
   - E2E tests (if permission granted)
   - Latency benchmarks (always run)
   - Manual test checklist prompt (if interactive terminal)
2. WHEN mock tests fail THEN script SHALL exit with non-zero status and clear error message
3. WHEN Accessibility permission is missing THEN script SHALL:
   - Print warning (not error)
   - Continue to benchmarks
   - Return exit code 0 (success)
4. WHEN running in non-interactive mode (CI) THEN script SHALL skip manual test prompts
5. WHEN running in interactive mode THEN script SHALL prompt user to run manual tests with y/N option
6. WHEN displaying progress THEN script SHALL use clear visual indicators (✅, ⚠️, ❌) for each test phase
7. WHEN tests complete THEN script SHALL print summary with:
   - Total tests run
   - Coverage percentage
   - Benchmark results (median latency)
   - Manual testing recommendations

### Requirement 4: Test Coverage Parity

**User Story:** As a platform maintainer, I want macOS test coverage to match Linux/Windows, so that all three platforms have equivalent reliability guarantees.

#### Acceptance Criteria

1. WHEN comparing test coverage THEN macOS SHALL have:
   - Unit tests: ≥ Linux coverage (currently 5 files with #[cfg(test)])
   - Integration tests: ≥ 400 lines (currently 430, maintain/expand)
   - E2E tests: Match Linux/Windows patterns (multidevice, tap-hold, basic remap)
2. WHEN running test suite THEN macOS SHALL test all critical paths:
   - CGEvent capture pipeline
   - CGEvent injection pipeline
   - IOKit device enumeration
   - Serial number extraction
   - Permission checking
   - Platform trait implementation
3. WHEN validating correctness THEN tests SHALL verify:
   - Cross-platform behavior consistency (same config = same output)
   - macOS-specific key mappings (Cmd, Option, Fn)
   - Accessibility API integration
   - System tray integration

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Test utilities should be focused and reusable (e.g., `MacosE2EHarness` handles only E2E setup/teardown)
- **Modular Design**: Permission checks, daemon spawning, and test assertions should be separate concerns
- **Dependency Management**: E2E harness should not depend on specific test cases
- **Clear Interfaces**: Test helper functions should have well-defined contracts

### Performance
- Mock tests must complete in <10 seconds total (CI efficiency)
- E2E tests should complete in <60 seconds per test (developer experience)
- Benchmarks should run for ≥1000 iterations to ensure statistical validity
- Test runner script overhead should be <1 second

### Reliability
- Tests must be deterministic (no flaky tests)
- E2E tests must clean up daemon processes even on test failure
- Permission checks must never hang or require user interaction
- Test runner must handle Ctrl+C gracefully (terminate all spawned processes)

### Usability
- Error messages must clearly indicate whether issue is test failure or environment problem
- Permission error messages must include copy-pasteable path to System Settings
- Test output must be machine-parseable (for CI log analysis)
- Test runner script must have --help flag with usage examples

### Maintainability
- Test code should follow same 500-line file size limit as production code
- Test utilities should be documented with usage examples
- E2E test harness should match patterns from Linux/Windows implementations
- Test coverage gaps should be automatically reported in CI
