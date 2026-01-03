# WebSocket Mock Infrastructure Guide

## ⚠️ DEPRECATED

This custom WebSocket mock has been replaced with **jest-websocket-mock**, an industry-standard library.

## Migration Guide

**Old Documentation**: This file (deprecated)
**New Documentation**: See `WEBSOCKET_TESTING.md`

### Quick Migration

**Before (Custom Mock):**
```typescript
import { getWebSocketMock, resetWebSocketMock } from '../tests/testUtils';

test('component test', () => {
  renderWithProviders(<MyComponent />);

  const ws = getWebSocketMock();
  ws?.simulateConnectedHandshake();
  ws?.simulateMessage({ type: 'event', data: {} });
});
```

**After (jest-websocket-mock):**
```typescript
import {
  setupMockWebSocket,
  cleanupMockWebSocket,
  simulateConnected,
  sendServerMessage,
} from '../tests/testUtils';

beforeEach(async () => await setupMockWebSocket());
afterEach(() => cleanupMockWebSocket());

test('component test', async () => {
  renderWithProviders(<MyComponent />);

  await simulateConnected();
  sendServerMessage({ type: 'event', data: {} });
});
```

## Why We Migrated

1. **Industry Standard**: jest-websocket-mock is battle-tested with 2.7k+ stars
2. **Better React Integration**: Automatic `act()` wrapping
3. **Custom Matchers**: `toReceiveMessage` and `toHaveReceivedMessages`
4. **Easier Maintenance**: No custom code to maintain
5. **Better Documentation**: Extensive examples and community support

## See New Documentation

📖 **[WEBSOCKET_TESTING.md](./WEBSOCKET_TESTING.md)** - Complete guide to the new infrastructure

---

**Last Updated**: 2026-01-03
**Status**: DEPRECATED - Use WEBSOCKET_TESTING.md instead
