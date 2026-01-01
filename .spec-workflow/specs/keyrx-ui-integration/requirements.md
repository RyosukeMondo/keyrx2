# Requirements Document

## Introduction

This specification defines the integration of keyrx_ui and keyrx_ui_v2 into a unified web interface for the KeyRx keyboard remapping system. The project merges keyrx_ui_v2's superior architecture (React Router, TailwindCSS, comprehensive testing) with keyrx_ui's Monaco code editor and real-time WebSocket dashboard features.

Additionally, this specification introduces a **unified WebSocket RPC API** that replaces the current dual REST + WebSocket architecture with a single, type-safe RPC protocol, simplifying client-server communication and enabling real-time-by-default operations.

The unified UI serves as the primary interface for:
- **Configuration management**: Visual keyboard editor + Monaco code editor with Rhai syntax highlighting
- **Real-time monitoring**: Live daemon state, event timeline, latency metrics via WebSocket
- **Profile management**: CRUD operations for keyboard configuration profiles
- **Device management**: Multi-device configuration with serial number-based identification
- **Simulation**: Browser-based WASM simulation for configuration testing

## Alignment with Product Vision

This feature aligns with keyrx's **"AI Coding Agent First"** principle and core objectives:

1. **AI-First Verification**: Monaco editor with WASM validation enables AI agents to verify Rhai configurations programmatically without human UAT
   - Real-time validation with line markers and quick fixes
   - Deterministic WASM simulation in browser matches daemon behavior byte-for-byte
   - Configuration errors caught at compile-time before deployment

2. **Single Source of Truth (SSOT)**: Unified WebSocket RPC API eliminates configuration drift
   - Single `.krx` binary consumed by daemon, UI, and tests
   - Hash-based verification for AI agents to confirm "configuration A == configuration B"
   - Type-safe end-to-end (Rust types → JSON schema → TypeScript types)

3. **Real-Time Observability**: Live dashboard provides machine-parseable state for AI monitoring
   - WebSocket event streaming (daemon state, key events, latency metrics)
   - JSON-formatted logs compatible with AI parsing
   - Correlation IDs for request/event tracing

4. **<1ms Latency Processing**: Real-time dashboard verifies sub-millisecond performance targets
   - Live latency metrics (P50/P95/P99)
   - Event timeline with microsecond timestamps
   - Performance regression detection via dashboard monitoring

5. **Deterministic Behavior**: WASM simulation ensures same input → same output for automated testing
   - Virtual clock for time-independent tests
   - Property-based testing integration
   - AI agents can verify correctness via deterministic tests

## Requirements

### Requirement 1: Unified WebSocket RPC API

**User Story:** As a **developer**, I want a **single WebSocket API** for all daemon communication, so that **I can simplify client state management, reduce connection overhead, and enable real-time-by-default operations**.

#### Acceptance Criteria

1. WHEN client connects to `ws://localhost:9867/api` THEN daemon SHALL upgrade HTTP to WebSocket and send "connected" handshake message
2. WHEN client sends `query` message with method "getProfiles" THEN daemon SHALL respond with `response` message containing profile list
3. WHEN client sends `command` message with method "createProfile" THEN daemon SHALL create profile and respond with success/error
4. WHEN client sends `subscribe` message with channel "daemon-state" THEN daemon SHALL broadcast state updates to client in real-time
5. WHEN client sends message with unknown method THEN daemon SHALL respond with error code -32601 ("Method not found")
6. WHEN multiple clients send concurrent requests with different IDs THEN daemon SHALL correlate responses via request ID matching
7. WHEN client disconnects THEN daemon SHALL clean up subscriptions and pending requests for that client
8. WHEN daemon broadcasts event to channel THEN all subscribed clients SHALL receive event message
9. IF request times out after 30 seconds THEN client SHALL reject promise with timeout error
10. WHEN client unsubscribes from channel THEN daemon SHALL stop sending events to that client for that channel

**RPC Methods:**
- Profile operations: `getProfiles`, `createProfile`, `activateProfile`, `deleteProfile`, `duplicateProfile`, `renameProfile`
- Device operations: `getDevices`, `renameDevice`, `setScopeDevice`, `forgetDevice`
- Config operations: `getConfig`, `updateConfig`, `setKeyMapping`, `deleteKeyMapping`, `getLayers`
- Metrics operations: `getLatency`, `getEvents`, `clearEvents`
- Simulator operations: `simulate`, `resetSimulator`

**Subscription Channels:**
- `daemon-state`: Emits `{ modifiers, locks, layer }` on state changes
- `events`: Emits `{ timestamp, keyCode, eventType, latency }` for each key event
- `latency`: Emits `{ min, avg, max, p95, p99, count }` performance metrics

### Requirement 2: Monaco Code Editor Integration

**User Story:** As a **power user**, I want a **professional code editor with Rhai syntax highlighting**, so that **I can edit keyboard configurations with IDE-like features (autocomplete, error detection, quick fixes)**.

#### Acceptance Criteria

1. WHEN user switches to "Code Editor" tab THEN ConfigPage SHALL render Monaco editor with Rhai language registered
2. WHEN user types Rhai code THEN Monaco editor SHALL highlight syntax with language-specific colors
3. WHEN user saves configuration with validation errors THEN editor SHALL prevent save and display error count in status bar
4. WHEN validation completes THEN editor SHALL display error markers at error line numbers with red squiggly underlines
5. WHEN user presses F8 THEN editor SHALL jump cursor to next error location and reveal line in center of viewport
6. WHEN user hovers over error marker THEN editor SHALL display error message tooltip
7. WHEN WASM validation is unavailable THEN editor SHALL display "Validation unavailable" status and allow save without validation
8. WHEN user edits code THEN validation SHALL debounce for 500ms before running WASM validation
9. IF WASM validation succeeds with zero errors THEN status bar SHALL display "✓ No issues found"
10. WHEN user clicks lightbulb icon THEN editor SHALL display quick fix suggestions for current error

### Requirement 3: Real-Time Dashboard with WebSocket

**User Story:** As a **developer/tester**, I want a **real-time monitoring dashboard**, so that **I can observe daemon state, event processing, and latency metrics live without polling**.

#### Acceptance Criteria

1. WHEN user navigates to `/dashboard` THEN DashboardPage SHALL connect to WebSocket and display connection status banner
2. WHEN WebSocket connects THEN connection banner SHALL show "🟢 Connected to daemon" in green
3. WHEN WebSocket disconnects THEN connection banner SHALL show "🔴 Disconnected" in red and auto-reconnect after 3 seconds
4. WHEN daemon broadcasts `state` event THEN StateIndicatorPanel SHALL update modifier badges (blue), lock badges (orange), and layer badge (green)
5. WHEN daemon broadcasts `latency` event THEN MetricsChart SHALL plot avg/P95/P99 lines in 60-second rolling window
6. WHEN daemon broadcasts `event` event THEN DashboardEventTimeline SHALL append to virtualized event list (max 100 events, FIFO)
7. WHEN user clicks "⏸️ Pause" THEN event timeline SHALL stop updating until resumed
8. WHEN user hovers over event in timeline THEN tooltip SHALL display full event details (timestamp, keyCode, eventType, latency)
9. IF event timeline exceeds 100 events THEN oldest events SHALL be removed (FIFO queue)
10. WHEN dashboard loads THEN MetricsChart SHALL display 5ms threshold reference line for performance target

### Requirement 4: Tab-Based Configuration Editor

**User Story:** As a **user**, I want to **switch between visual keyboard editor and code editor**, so that **I can choose the editing mode that fits my workflow (visual for simple changes, code for complex logic)**.

#### Acceptance Criteria

1. WHEN user opens ConfigPage THEN "Visual Editor" tab SHALL be active by default
2. WHEN user clicks "Code Editor" tab THEN Monaco editor SHALL render with current configuration code
3. WHEN user clicks "Visual Editor" tab THEN KeyboardVisualizer SHALL render with current keyboard layout
4. WHEN user switches tabs THEN active tab button SHALL display with primary background color and white text
5. WHEN user edits in Code Editor and switches tabs THEN changes SHALL persist across tab switches
6. WHEN validation errors exist THEN both tabs SHALL show validation status panel with error count
7. WHEN user saves from either tab THEN same save operation SHALL execute (unified logic)
8. IF user has unsaved changes and switches tabs THEN changes SHALL not be lost (no automatic revert)
9. WHEN validation completes THEN validation status panel SHALL update in real-time regardless of active tab
10. WHEN user presses Ctrl+S (or Cmd+S) in either tab THEN save SHALL trigger if no validation errors

### Requirement 5: WASM Integration for Validation and Simulation

**User Story:** As an **AI coding agent**, I want **WASM-based configuration validation**, so that **I can verify Rhai configurations programmatically without daemon deployment**.

#### Acceptance Criteria

1. WHEN UI builds THEN WASM module SHALL be built from keyrx_core with `wasm32-unknown-unknown` target
2. WHEN Monaco editor loads THEN WASM module SHALL initialize via `wasmCore.init()` before validation
3. WHEN user types invalid Rhai code THEN WASM `validate_config()` SHALL return validation errors with line/column numbers
4. WHEN WASM validation runs THEN result SHALL match daemon behavior byte-for-byte (deterministic)
5. WHEN WASM build fails THEN UI SHALL gracefully disable validation and show "Validation unavailable" status
6. IF WASM module fails to load THEN Monaco editor SHALL still function but without real-time validation
7. WHEN user triggers simulation THEN WASM `simulate()` SHALL execute identical core logic as daemon
8. WHEN WASM simulation runs THEN results SHALL include state transitions, output events, and latency estimates
9. WHEN Rust source in keyrx_core changes during development THEN Vite SHALL auto-rebuild WASM and hot-reload page
10. WHEN production build runs THEN WASM module SHALL be optimized with `--release` flag and stripped of debug symbols

### Requirement 6: Responsive UI with TailwindCSS

**User Story:** As a **user**, I want a **responsive UI that works on mobile, tablet, and desktop**, so that **I can manage keyboard configurations from any device**.

#### Acceptance Criteria

1. WHEN viewport width < 768px THEN BottomNav SHALL render with navigation icons
2. WHEN viewport width >= 768px THEN Sidebar SHALL render with fixed left navigation
3. WHEN viewport width < 768px THEN ConfigPage SHALL use single-column layout
4. WHEN viewport width >= 1024px THEN ConfigPage SHALL use multi-column grid layout
5. WHEN user accesses dashboard on mobile THEN connection banner, state indicators, chart, and timeline SHALL stack vertically
6. WHEN user accesses dashboard on desktop THEN state indicators and chart SHALL render in 2-column grid
7. WHEN buttons render on mobile THEN minimum tap target SHALL be 44x44px for accessibility
8. WHEN visual regression tests run THEN screenshots SHALL be captured at 3 breakpoints (mobile, tablet, desktop)
9. IF component has separate CSS file THEN CSS SHALL be converted to TailwindCSS utility classes
10. WHEN production build runs THEN unused TailwindCSS classes SHALL be purged for minimal bundle size

### Requirement 7: Comprehensive Testing Infrastructure

**User Story:** As a **maintainer**, I want **comprehensive automated testing**, so that **I can ensure code quality, catch regressions, and verify accessibility without manual testing**.

#### Acceptance Criteria

1. WHEN unit tests run THEN code coverage SHALL be ≥ 80% overall
2. WHEN unit tests run for keyrx_core THEN code coverage SHALL be ≥ 90% (critical path)
3. WHEN integration tests run THEN tab switching, validation flow, and WebSocket updates SHALL be verified
4. WHEN E2E tests run THEN Playwright SHALL test full workflows (create profile, edit config, activate, monitor dashboard)
5. WHEN accessibility tests run THEN Playwright with axe-core SHALL verify zero WCAG violations
6. WHEN performance tests run THEN bundle size SHALL be verified < 500KB initial load (gzipped)
7. WHEN visual regression tests run THEN Playwright SHALL capture screenshots and compare against baselines
8. WHEN pre-commit hook runs THEN clippy, rustfmt, and tests SHALL all pass before commit
9. IF any test fails THEN CI SHALL fail build and prevent merge
10. WHEN new component is added THEN corresponding test file SHALL be created before merge

### Requirement 8: Build Process Integration

**User Story:** As a **developer**, I want an **automated build process**, so that **WASM, UI, and daemon are built in the correct order with verification at each step**.

#### Acceptance Criteria

1. WHEN `npm run build:wasm` runs THEN keyrx_core SHALL compile to WASM and output to `src/wasm/pkg/`
2. WHEN `npm run build` runs THEN WASM SHALL build first, followed by TypeScript compilation and Vite bundle
3. WHEN daemon builds THEN keyrx_ui_v2/dist SHALL be embedded via `include_dir!` macro at compile-time
4. WHEN WASM build fails THEN UI build SHALL fail with clear error message
5. WHEN UI build completes THEN `dist/` directory SHALL contain index.html and assets/
6. WHEN daemon build runs without UI dist THEN build SHALL fail with "UI directory not found" error
7. WHEN `make build` runs THEN WASM → UI → daemon SHALL execute in sequence
8. WHEN `make verify` runs THEN clippy, fmt, tests, and coverage SHALL all pass
9. IF daemon embeds UI successfully THEN unit test SHALL verify `UI_DIR.get_file("index.html")` exists
10. WHEN CI runs THEN full build sequence SHALL execute and all tests SHALL pass before deployment

## Non-Functional Requirements

### Code Architecture and Modularity

**Code Quality (per CLAUDE.md):**
- **File size limit**: Maximum 500 lines per file (excluding comments/blank lines)
- **Function size limit**: Maximum 50 lines per function
- **Strict TypeScript**: All TypeScript must use strict mode with no `any` types
- **Component documentation**: All props must be documented with TypeScript types
- **Dependency injection**: All external dependencies (APIs, WebSocket, storage) must be injectable for testing
- **Separation of concerns**: Components, hooks, stores, and utilities must be in separate directories

**Architecture patterns:**
- **SOLID principles**: Single responsibility, Open/closed, Liskov substitution, Interface segregation, Dependency inversion
- **SSOT**: Single source of truth for configuration (`.krx` binary)
- **KISS**: Keep it simple - no premature optimization or over-engineering
- **DI**: Dependency injection for all external dependencies (API endpoints, WebSocket, storage)

### Performance

**Hard Requirements:**
- Initial bundle load: < 500KB (gzipped)
- Monaco editor chunk: < 2MB (lazy-loaded)
- WASM module: < 1MB (lazy-loaded)
- Monaco editor initialization: < 1 second
- WebSocket connection: < 500ms
- Dashboard real-time updates: < 100ms latency
- WASM validation: < 200ms for typical configuration

**Optimization strategies:**
- Code splitting by feature library (Monaco, Recharts, React Router in separate chunks)
- Virtual scrolling for event timeline (react-window)
- Debounced validation (500ms)
- Lazy loading for Monaco and WASM
- Production build with Terser minification, Gzip, and Brotli compression

### Security

**Requirements:**
- No secret logging: Configuration content may contain sensitive data, only log metadata (hash, size)
- Input validation: All RPC parameters validated before processing
- WebSocket origin validation: Only localhost connections allowed (no remote access)
- WASM sandbox: WASM module runs in browser sandbox with no filesystem/network access
- Content Security Policy: Restrict inline scripts and external resources

**Threat model:**
- Malicious configuration injection: Validated by WASM before daemon deployment
- XSS attacks: React automatically escapes user input
- CSRF attacks: Not applicable (localhost-only WebSocket)

### Reliability

**Availability:**
- WebSocket auto-reconnect: 3-second intervals with exponential backoff
- Graceful degradation: UI functions without WASM (validation disabled)
- Error boundaries: React error boundaries catch component crashes
- State recovery: Local storage persistence for user preferences (tab selection, layout)

**Error handling:**
- All API errors logged to console with structured format
- User-facing errors displayed with actionable messages
- WebSocket reconnection attempts logged with retry count
- Validation errors shown with line/column numbers and quick fix suggestions

### Usability

**Accessibility:**
- WCAG AA compliance: All interactive elements meet color contrast requirements
- Keyboard navigation: All features accessible via keyboard shortcuts
- Screen reader support: ARIA labels on all interactive elements
- Focus management: Visible focus indicators, logical tab order

**User experience:**
- Loading states: Skeleton screens during data fetching
- Optimistic updates: UI updates immediately, rollback on error
- Responsive design: Mobile-first approach with 3 breakpoints
- Error feedback: Clear error messages with recovery suggestions
- Real-time feedback: Live validation, instant WebSocket updates

**Internationalization (future):**
- UI text externalized for i18n support
- Date/time formatting locale-aware
- Number formatting locale-aware

## Success Metrics

**Technical metrics:**
- ✅ All tests pass (unit, integration, E2E, visual, accessibility, performance)
- ✅ Code coverage ≥ 80% overall, ≥ 90% for keyrx_core
- ✅ Bundle size < 500KB initial load (gzipped)
- ✅ Lighthouse score ≥ 90 (performance, accessibility, best practices)
- ✅ Zero console errors in production build
- ✅ All pages responsive (mobile, tablet, desktop)

**Functional metrics:**
- ✅ Monaco editor validates Rhai code with WASM
- ✅ Real-time dashboard displays WebSocket updates
- ✅ Tab switching works in ConfigPage
- ✅ WebSocket reconnects automatically on disconnect
- ✅ Daemon successfully embeds and serves UI

**User experience metrics:**
- ✅ Configuration change time < 5 seconds (edit → save → deploy)
- ✅ WASM simulation matches daemon behavior byte-for-byte
- ✅ Validation errors caught before deployment
- ✅ Dashboard updates in real-time (< 100ms latency)
