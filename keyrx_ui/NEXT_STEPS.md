# Next Steps: Completing Test Infrastructure Migration

**Status as of 2026-01-03 (Updated 9:50 AM)**

## ✅ What's Been Accomplished

### Infrastructure Built (Phase 1 - COMPLETE)
1. **Industry-Standard WebSocket Testing** - jest-websocket-mock installed and configured
2. **Helper Functions** - Clean API in `tests/helpers/websocket.ts`
3. **Comprehensive Documentation** - Complete guide in `tests/WEBSOCKET_TESTING.md`
4. **Test Improvements** - Fixed ProfilesPage (19/19), DevicesPage (16/18)
5. **WebSocket Mock Configuration** - Made global.WebSocket writable for jest-websocket-mock
6. **react-use-websocket Mock** - Fixed assertIsWebSocket bypass to allow mock WebSocket

### Accessibility Tests Migration (Phase 1 - COMPLETE)
- ✅ **All 6 accessibility test files migrated to jest-websocket-mock**
- ✅ DashboardPage.a11y.test.tsx
- ✅ ConfigPage.a11y.test.tsx
- ✅ DevicesPage.a11y.test.tsx
- ✅ ProfilesPage.a11y.test.tsx
- ✅ MetricsPage.a11y.test.tsx
- ✅ SimulatorPage.a11y.test.tsx

### Current Test Status
- **729/897 tests passing (81.3%)**
- **Target**: 852/897 (≥95%)
- **Gap**: +123 tests needed
- **Progress**: +4 tests from baseline (725 → 729)
- **Errors**: 164 errors remaining (some WebSocket-related)

## 📋 Migration Checklist

### Phase 1: Component Tests with WebSocket (Priority 1)

Tests that render components using `useUnifiedApi` hook need WebSocket mock setup:

```typescript
// Pattern to apply:
import {
  setupMockWebSocket,
  cleanupMockWebSocket,
  simulateConnected
} from '../tests/testUtils';

beforeEach(async () => {
  await setupMockWebSocket();
});

afterEach(() => {
  cleanupMockWebSocket();
});

test('component test', async () => {
  renderWithProviders(<Component />);
  await simulateConnected();  // Wait for handshake
  // ... rest of test
});
```

**Files to Migrate:**

- [ ] `src/pages/SimulatorPage.a11y.test.tsx`
- [ ] `src/pages/DashboardPage.a11y.test.tsx`
- [ ] `src/pages/MetricsPage.a11y.test.tsx`
- [ ] `src/pages/ConfigPage.a11y.test.tsx`
- [ ] Any other *.a11y.test.tsx files

**Estimated Impact**: +20-30 tests passing

### Phase 2: Integration Tests (Priority 2)

Integration tests that test WebSocket communication:

- [ ] `tests/integration/dashboard-updates.test.tsx`
- [ ] `tests/integration/rpc-communication.test.ts`
- [ ] `tests/integration/profile-workflow.test.tsx`
- [ ] `tests/integration/config-editor.test.tsx`

**Estimated Impact**: +15-25 tests passing

### Phase 3: ConfigPage Component Tests (Priority 3)

Apply same async pattern as ProfilesPage:

- [ ] `src/pages/ConfigPage.test.tsx`
- [ ] `src/pages/ConfigPage.integration.test.tsx`

**Estimated Impact**: +10-20 tests passing

### Phase 4: E2E Tests (Evaluate)

Determine if Playwright e2e tests should count toward 95% metric:

- [ ] `e2e/*.spec.ts`
- [ ] `tests/e2e/*.spec.ts`

**Decision needed**: Are these in scope for unit/integration test target?

## 🔧 Migration Steps (Detailed)

### For Components Using WebSocket

**1. Identify the Component**
- Does it use `useUnifiedApi` hook?
- Does it need real-time updates?

**2. Update Test Structure**
```typescript
// Add imports
import {
  setupMockWebSocket,
  cleanupMockWebSocket,
  simulateConnected,
  sendDaemonStateUpdate,
  sendLatencyUpdate,
} from '../../../tests/testUtils';

// Add setup/teardown
beforeEach(async () => {
  await setupMockWebSocket();
});

afterEach(() => {
  cleanupMockWebSocket();
});
```

**3. Update Individual Tests**
```typescript
// Add async and wait for connection
test('my test', async () => {  // Make it async
  renderWithProviders(<Component />);

  // Wait for WebSocket handshake
  await simulateConnected();

  // Send updates as needed
  sendDaemonStateUpdate({ running: true });

  // Assert using findBy or waitFor
  await screen.findByText('Running');
});
```

**4. Handle Component-Specific Needs**

If component subscribes to specific channels:
```typescript
await simulateConnected();

// Component subscribes to daemon-state
sendDaemonStateUpdate({
  running: true,
  activeProfile: 'default'
});

// Component subscribes to latency
sendLatencyUpdate({
  avg: 1.2,
  min: 0.5,
  max: 3.8
});
```

### For Accessibility Tests

Accessibility tests often fail because components don't render without WebSocket data:

```typescript
test('accessibility test', async () => {
  await setupMockWebSocket();  // Before render

  const { container } = renderWithProviders(<Component />);

  await simulateConnected();

  // Send required data
  sendDaemonStateUpdate({ running: true });

  // Wait for component to fully render
  await screen.findByRole('main');  // Or other landmark

  // Now run accessibility audit
  const results = await axe(container);
  expect(results).toHaveNoViolations();
});

afterEach(() => {
  cleanupMockWebSocket();
});
```

## 🎯 Quick Wins

### Start with SimulatorPage Accessibility Test

This is currently failing and would be a good pilot:

```typescript
// src/pages/SimulatorPage.a11y.test.tsx
import {
  setupMockWebSocket,
  cleanupMockWebSocket,
  simulateConnected
} from '../../tests/testUtils';

describe('SimulatorPage Accessibility', () => {
  beforeEach(async () => {
    await setupMockWebSocket();
  });

  afterEach(() => {
    cleanupMockWebSocket();
  });

  it('should pass complete accessibility audit', async () => {
    const { container } = renderWithProviders(<SimulatorPage />);

    // Wait for WebSocket connection
    await simulateConnected();

    // Wait for page to render
    await screen.findByRole('main', {}, { timeout: 3000 });

    // Run accessibility audit
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have proper page structure with landmarks', async () => {
    const { container } = renderWithProviders(<SimulatorPage />);

    await simulateConnected();
    await screen.findByRole('main');

    // Verify semantic HTML landmarks exist
    const main = container.querySelector('main');
    expect(main).toBeTruthy();
  });
});
```

## 📊 Expected Results

After completing all phases:

| Phase | Tests Fixed | Cumulative | Pass Rate |
|-------|-------------|------------|-----------|
| Starting | - | 725/897 | 80.9% |
| Phase 1 (A11y) | +25 | 750/897 | 83.6% |
| Phase 2 (Integration) | +20 | 770/897 | 85.8% |
| Phase 3 (ConfigPage) | +15 | 785/897 | 87.5% |
| Remaining Fixes | +67 | 852/897 | **95.0%** ✅ |

## 🚨 Common Pitfalls

### 1. Forgetting to Setup WebSocket
```typescript
// ❌ BAD - Component won't render properly
test('test', () => {
  renderWithProviders(<Component />);
  // Component stuck in loading state
});

// ✅ GOOD
beforeEach(async () => await setupMockWebSocket());
test('test', async () => {
  renderWithProviders(<Component />);
  await simulateConnected();
});
```

### 2. Not Waiting for Connection
```typescript
// ❌ BAD - Sending before connection ready
test('test', async () => {
  await setupMockWebSocket();
  sendDaemonStateUpdate({ running: true });  // Too early!
  renderWithProviders(<Component />);
});

// ✅ GOOD
test('test', async () => {
  await setupMockWebSocket();
  renderWithProviders(<Component />);
  await simulateConnected();  // Wait first
  sendDaemonStateUpdate({ running: true });
});
```

### 3. Forgetting Cleanup
```typescript
// ❌ BAD - WebSocket persists across tests
beforeEach(async () => await setupMockWebSocket());
// Missing afterEach!

// ✅ GOOD
beforeEach(async () => await setupMockWebSocket());
afterEach(() => cleanupMockWebSocket());
```

## 📖 Resources

- **Complete API Reference**: `tests/WEBSOCKET_TESTING.md`
- **Infrastructure Summary**: `TEST_INFRASTRUCTURE_IMPROVEMENTS.md`
- **Helper Code**: `tests/helpers/websocket.ts`
- **Example**: ProfilesPage.test.tsx (successful migration reference)

## 🎬 Getting Started

**Recommended First Task:**

1. Open `src/pages/SimulatorPage.a11y.test.tsx`
2. Add WebSocket setup following the pattern above
3. Run test: `npm test -- SimulatorPage.a11y.test.tsx`
4. Verify it passes
5. Use as template for other a11y tests

**Commands:**
```bash
# Run specific test file
npm test -- SimulatorPage.a11y.test.tsx

# Run all a11y tests
npm test -- a11y

# Check overall progress
npm test 2>&1 | tail -5
```

## 🎯 Success Criteria

- [ ] All accessibility tests passing (23/23)
- [ ] Test pass rate ≥95% (852/897)
- [ ] Zero WebSocket-related errors
- [ ] All component tests use new infrastructure
- [ ] Old custom mock removed

## 💡 Tips

1. **Start Small**: Migrate one test file at a time
2. **Test Frequently**: Run tests after each migration
3. **Follow Patterns**: Use ProfilesPage.test.tsx as reference
4. **Read Docs**: Check `WEBSOCKET_TESTING.md` when stuck
5. **Ask for Help**: File in documentation is comprehensive

---

**Last Updated**: 2026-01-03
**Next Review**: After Phase 1 completion
