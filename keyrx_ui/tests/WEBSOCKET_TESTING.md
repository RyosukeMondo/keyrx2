# WebSocket Testing Infrastructure

## Overview

This guide explains the robust WebSocket testing infrastructure using **jest-websocket-mock** (compatible with Vitest).

### Why jest-websocket-mock?

- ✅ **Industry Standard**: 2.7k+ GitHub stars, actively maintained
- ✅ **React Integration**: Automatically wraps calls in `act()` when React Testing Library is present
- ✅ **Custom Matchers**: Provides `toReceiveMessage` and `toHaveReceivedMessages` for assertions
- ✅ **JSON Protocol**: Built-in serialization/deserialization support
- ✅ **Vitest Compatible**: Works seamlessly with Vitest despite "jest" in the name
- ✅ **Mock Socket Foundation**: Uses the reliable `mock-socket` library under the hood

## Installation

Already installed in this project:

```bash
npm install --save-dev jest-websocket-mock mock-socket
```

##Quick Start

### Basic Test Pattern

```typescript
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/react';
import {
  renderWithProviders,
  setupMockWebSocket,
  cleanupMockWebSocket,
  simulateConnected,
  sendDaemonStateUpdate,
} from '../tests/testUtils';
import { DashboardPage } from './DashboardPage';

describe('DashboardPage', () => {
  beforeEach(async () => {
    // Setup mock WebSocket server before each test
    await setupMockWebSocket();
  });

  afterEach(() => {
    // Clean up WebSocket connections after each test
    cleanupMockWebSocket();
  });

  it('displays daemon state updates', async () => {
    renderWithProviders(<DashboardPage />);

    // Simulate connection handshake
    await simulateConnected('test-session-123');

    // Send daemon state update from server
    sendDaemonStateUpdate({
      running: true,
      activeProfile: 'default',
    });

    // Assert UI updated
    await screen.findByText('Running');
    expect(screen.getByText('Profile: default')).toBeInTheDocument();
  });
});
```

## API Reference

### Setup and Teardown

#### `setupMockWebSocket(url?, options?): Promise<WS>`

Creates and initializes a mock WebSocket server.

**Parameters:**
- `url` (optional): WebSocket URL (default: `'ws://localhost:3030/ws'`)
- `options` (optional): Configuration options
  - `jsonProtocol` (boolean): Enable automatic JSON serialization (default: `true`)

**Returns:** Promise resolving to the mock server instance

**Example:**
```typescript
beforeEach(async () => {
  await setupMockWebSocket();
});
```

#### `cleanupMockWebSocket(): void`

Closes all WebSocket connections and cleans up the mock server.

**Example:**
```typescript
afterEach(() => {
  cleanupMockWebSocket();
});
```

### Server Control

#### `getMockWebSocket(): WS`

Returns the current mock server instance for advanced control.

**Throws:** Error if server not initialized

**Example:**
```typescript
const server = getMockWebSocket();
await server.connected; // Wait for client connection
server.close(); // Close connection
```

#### `sendServerMessage(message: any): void`

Sends a message from server to all connected clients. Objects are automatically serialized to JSON.

**Example:**
```typescript
sendServerMessage({
  type: 'response',
  id: '123',
  result: { success: true },
});
```

### Helper Functions

#### `simulateConnected(sessionId?): Promise<void>`

Simulates the connection handshake from the server.

**Parameters:**
- `sessionId` (optional): Session ID (default: `'test-session'`)

**Example:**
```typescript
await simulateConnected('my-session-123');
```

#### `sendDaemonStateUpdate(state: Record<string, unknown>): void`

Sends a daemon state update event.

**Example:**
```typescript
sendDaemonStateUpdate({
  running: true,
  activeProfile: 'gaming',
  connectedDevices: 2,
});
```

#### `sendLatencyUpdate(stats: Record<string, unknown>): void`

Sends latency statistics update.

**Example:**
```typescript
sendLatencyUpdate({
  avg: 1.2,
  min: 0.5,
  max: 3.8,
  p50: 1.1,
  p95: 2.5,
  p99: 3.2,
});
```

### Message Assertions

#### `waitForMessage(expectedMessage: any, timeout?): Promise<void>`

Waits for the server to receive a specific message from the client.

**Parameters:**
- `expectedMessage`: Expected message content (partial match supported)
- `timeout` (optional): Timeout in milliseconds (default: `1000`)

**Example:**
```typescript
// Wait for subscribe message
await waitForMessage({
  type: 'subscribe',
  channel: 'daemon-state',
});

// Wait for query with specific method
await waitForMessage({
  type: 'query',
  method: 'getProfiles',
});
```

#### `assertReceivedMessages(expectedMessages: any[]): void`

Asserts that the server has received specific messages (synchronous, checks history).

**Example:**
```typescript
assertReceivedMessages([
  { type: 'subscribe', channel: 'daemon-state' },
  { type: 'subscribe', channel: 'latency' },
  { type: 'query', method: 'getProfiles' },
]);
```

## Advanced Usage

### Custom Matchers

jest-websocket-mock provides custom Vitest matchers:

```typescript
import { getMockWebSocket } from '../tests/testUtils';

test('receives messages', async () => {
  const server = getMockWebSocket();

  // Async matcher - waits for next message (1000ms timeout)
  await expect(server).toReceiveMessage({ type: 'subscribe' });

  // Synchronous matcher - checks received history
  expect(server).toHaveReceivedMessages([
    { type: 'subscribe', channel: 'daemon-state' },
    { type: 'subscribe', channel: 'latency' },
  ]);
});
```

### Testing Error Scenarios

```typescript
test('handles connection errors', async () => {
  await setupMockWebSocket();
  renderWithProviders(<MyComponent />);

  const server = getMockWebSocket();

  // Simulate server error
  server.error();

  // Assert error handling
  await screen.findByText(/connection error/i);
});
```

### Testing Connection Close

```typescript
test('handles connection close', async () => {
  await setupMockWebSocket();
  renderWithProviders(<MyComponent />);

  const server = getMockWebSocket();

  // Simulate graceful close
  server.close();

  // Assert reconnection attempt
  await screen.findByText(/reconnecting/i);
});
```

### Multiple Message Sequences

```typescript
test('handles complex message flow', async () => {
  await setupMockWebSocket();
  renderWithProviders(<ConfigPage />);

  // 1. Handshake
  await simulateConnected();

  // 2. Initial state
  sendDaemonStateUpdate({ running: false });
  await screen.findByText('Stopped');

  // 3. State change
  sendDaemonStateUpdate({ running: true });
  await screen.findByText('Running');

  // 4. Assert client sent subscription
  await waitForMessage({
    type: 'subscribe',
    channel: 'daemon-state',
  });
});
```

## Testing Hooks That Use WebSocket

### Testing useUnifiedApi Hook

```typescript
test('useUnifiedApi subscribes to channels', async () => {
  await setupMockWebSocket();

  const { result } = renderHook(() => useUnifiedApi(), {
    wrapper: ({ children }) => (
      <QueryClientProvider client={queryClient}>
        {children}
      </QueryClientProvider>
    ),
  });

  // Wait for connection
  await simulateConnected();

  // Assert subscriptions
  assertReceivedMessages([
    { type: 'subscribe', channel: 'daemon-state' },
    { type: 'subscribe', channel: 'latency' },
  ]);

  // Verify state updates
  sendDaemonStateUpdate({ running: true });
  await waitFor(() => {
    expect(result.current.daemonState.running).toBe(true);
  });
});
```

## Migration from Custom Mock

### Before (Custom Mock)

```typescript
import { getWebSocketMock } from '../tests/testUtils';

test('old approach', () => {
  renderWithProviders(<Component />);

  const ws = getWebSocketMock();
  ws?.simulateConnectedHandshake();
  ws?.simulateMessage({ type: 'event', data: {} });

  const sent = ws?.getSentMessages();
  expect(sent).toContain('some message');
});
```

### After (jest-websocket-mock)

```typescript
import {
  setupMockWebSocket,
  cleanupMockWebSocket,
  simulateConnected,
  sendServerMessage,
  waitForMessage,
} from '../tests/testUtils';

beforeEach(async () => await setupMockWebSocket());
afterEach(() => cleanupMockWebSocket());

test('new approach', async () => {
  renderWithProviders(<Component />);

  await simulateConnected();
  sendServerMessage({ type: 'event', data: {} });

  await waitForMessage({ type: 'subscribe' });
});
```

## Troubleshooting

### Test Hangs Waiting for Connection

**Problem:** Test times out waiting for WebSocket connection.

**Solution:** Ensure `setupMockWebSocket()` is called before rendering components:

```typescript
beforeEach(async () => {
  await setupMockWebSocket(); // Must be before render
});

test('my test', () => {
  renderWithProviders(<Component />);
  // ...
});
```

### Messages Not Received by Component

**Problem:** Component doesn't update after `sendServerMessage()`.

**Solution:** Wait for connection handshake first:

```typescript
await simulateConnected(); // Wait for handshake
sendDaemonStateUpdate({ running: true }); // Now this works
```

### Cannot Find WebSocket Server

**Problem:** `getMockWebSocket()` throws "not initialized" error.

**Solution:** Call `setupMockWebSocket()` in `beforeEach`:

```typescript
beforeEach(async () => {
  await setupMockWebSocket();
});
```

### jest.useFakeTimers() Breaks Tests

**Problem:** Using fake timers causes WebSocket to never connect.

**Issue:** mock-socket uses setTimeout internally, which is affected by fake timers.

**Solution:** Don't use `jest.useFakeTimers()` with WebSocket tests, or use real timers for WebSocket-related delays.

## Best Practices

1. **Always use beforeEach/afterEach**
   ```typescript
   beforeEach(async () => await setupMockWebSocket());
   afterEach(() => cleanupMockWebSocket());
   ```

2. **Wait for connection before sending messages**
   ```typescript
   await simulateConnected();
   sendDaemonStateUpdate({ ... });
   ```

3. **Use helper functions for common patterns**
   - `simulateConnected()` instead of manual handshake
   - `sendDaemonStateUpdate()` instead of raw `sendServerMessage()`

4. **Use custom matchers for assertions**
   ```typescript
   await waitForMessage({ type: 'subscribe' });
   // Better than: expect(server.getSentMessages()).toContain(...)
   ```

5. **Test isolation - don't share server instances**
   ```typescript
   // BAD - server persists across tests
   const server = setupMockWebSocket();

   // GOOD - fresh server for each test
   beforeEach(async () => await setupMockWebSocket());
   ```

## References

- [jest-websocket-mock GitHub](https://github.com/romgain/jest-websocket-mock)
- [jest-websocket-mock npm](https://www.npmjs.com/package/jest-websocket-mock)
- [mock-socket GitHub](https://github.com/thoov/mock-socket)
- [React WebSocket Testing Guide](https://wanago.io/2022/08/08/javascript-testing-mocking-websockets-mock-socket/)
