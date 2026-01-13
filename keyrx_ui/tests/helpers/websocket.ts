/**
 * WebSocket Testing Infrastructure using jest-websocket-mock
 *
 * This provides a robust WebSocket mock server that:
 * - Automatically integrates with @testing-library/react's act()
 * - Works seamlessly with react-use-websocket
 * - Provides custom matchers for assertions
 * - Handles message serialization/deserialization
 * - Compatible with Vitest
 *
 * Why jest-websocket-mock instead of MSW WebSocket:
 * - Proven compatibility with react-use-websocket
 * - Passes react-use-websocket's assertIsWebSocket check
 * - Simpler API for WebSocket-specific testing
 * - Better async handling for WebSocket lifecycle
 *
 * MSW is still used for HTTP mocking (proven to work well).
 * This hybrid approach uses the best tool for each job.
 */

import WS from 'jest-websocket-mock';
import { vi } from 'vitest';

/**
 * Default WebSocket URL for tests
 * Matches the URL used by useUnifiedApi hook
 */
export const WS_URL = 'ws://localhost:3030/ws';

/**
 * Global mock server instance
 * Initialized in test setup, cleaned up in teardown
 */
let mockServer: WS | null = null;

/**
 * Create and connect to a mock WebSocket server
 *
 * @param url - WebSocket URL (default: WS_URL)
 * @param options - Server configuration options
 * @returns Mock server instance
 *
 * @example
 * ```typescript
 * beforeEach(async () => {
 *   await setupMockWebSocket();
 * });
 * ```
 */
export async function setupMockWebSocket(
  url: string = WS_URL,
  options?: { jsonProtocol?: boolean }
): Promise<WS> {
  // Clean up any existing server
  if (mockServer) {
    try {
      WS.clean();
    } catch (err) {
      // Ignore errors during cleanup
    }
    mockServer = null;
  }

  // IMPORTANT: jsdom's WebSocket is read-only by default
  // We need to make it configurable so jest-websocket-mock can replace it
  if (typeof window !== 'undefined' && 'WebSocket' in window) {
    const originalWebSocket = window.WebSocket;
    Object.defineProperty(window, 'WebSocket', {
      writable: true,
      configurable: true,
      value: originalWebSocket,
    });
  }

  // Create new server with graceful JSON parsing that handles both JSON and non-JSON messages
  // This prevents "Unexpected token" errors when non-JSON strings are sent
  const safeDeserializer = (message: string | Buffer): any => {
    const messageStr = message.toString();
    try {
      return JSON.parse(messageStr);
    } catch {
      // If parsing fails, return the raw string
      // This allows tests to send both JSON and plain text
      return messageStr;
    }
  };

  const safeSerializer = (message: any): string => {
    if (typeof message === 'string') {
      return message;
    }
    return JSON.stringify(message);
  };

  mockServer = new WS(url, {
    deserializer: safeDeserializer, // Safe deserializer for incoming messages
    serializer: safeSerializer, // Safe serializer for outgoing messages
    ...options,
  });

  return mockServer;
}

/**
 * Get the current mock WebSocket server
 *
 * @returns Current mock server instance
 * @throws Error if server not initialized
 *
 * @example
 * ```typescript
 * const server = getMockWebSocket();
 * await server.connected; // Wait for connection
 * server.send({ type: 'connected', sessionId: 'test-123' });
 * ```
 */
export function getMockWebSocket(): WS {
  if (!mockServer) {
    throw new Error(
      'Mock WebSocket server not initialized. Call setupMockWebSocket() first.'
    );
  }
  return mockServer;
}

/**
 * Clean up mock WebSocket server
 * Should be called in afterEach to prevent test pollution
 *
 * @example
 * ```typescript
 * afterEach(() => {
 *   cleanupMockWebSocket();
 * });
 * ```
 */
export function cleanupMockWebSocket(): void {
  WS.clean();
  mockServer = null;
}

/**
 * Send a server message to all connected clients
 *
 * @param message - Message to send (auto-serialized if object)
 *
 * @example
 * ```typescript
 * sendServerMessage({ type: 'event', channel: 'daemon-state', data: { running: true } });
 * ```
 */
export function sendServerMessage(message: any): void {
  const server = getMockWebSocket();
  // jsonProtocol: true handles JSON serialization automatically
  server.send(message);
}

/**
 * Simulate a "connected" handshake from the server
 *
 * @param sessionId - Session ID to include in handshake
 *
 * @example
 * ```typescript
 * await simulateConnected('test-session-123');
 * ```
 */
export async function simulateConnected(sessionId: string = 'test-session'): Promise<void> {
  const server = getMockWebSocket();
  await server.connected; // Wait for client to connect
  server.send({
    type: 'connected',
    sessionId,
    timestamp: Date.now(),
  });
}

/**
 * Simulate a daemon state update event
 *
 * @param state - Daemon state data
 *
 * @example
 * ```typescript
 * sendDaemonStateUpdate({ running: true, activeProfile: 'default' });
 * ```
 */
export function sendDaemonStateUpdate(state: Record<string, unknown>): void {
  sendServerMessage({
    type: 'event',
    channel: 'daemon-state',
    data: state,
  });
}

/**
 * Simulate a latency stats update event
 *
 * @param stats - Latency statistics
 *
 * @example
 * ```typescript
 * sendLatencyUpdate({ avg: 1.2, min: 0.5, max: 3.8 });
 * ```
 */
export function sendLatencyUpdate(stats: Record<string, unknown>): void {
  sendServerMessage({
    type: 'event',
    channel: 'latency',
    data: stats,
  });
}

/**
 * Simulate a key event
 *
 * @param event - Key event data
 *
 * @example
 * ```typescript
 * sendKeyEvent({ keyCode: 'KEY_A', type: 'press', layer: 'base' });
 * ```
 */
export function sendKeyEvent(event: Record<string, unknown>): void {
  sendServerMessage({
    type: 'event',
    channel: 'key-events',
    data: event,
  });
}

/**
 * Wait for the server to receive a specific message
 *
 * @param expectedMessage - Expected message content
 * @param timeout - Timeout in milliseconds (default: 1000)
 *
 * @example
 * ```typescript
 * await waitForMessage({ type: 'query', method: 'getProfiles' });
 * ```
 */
export async function waitForMessage(
  expectedMessage: any,
  timeout: number = 1000
): Promise<void> {
  const server = getMockWebSocket();
  // Use the custom matcher provided by jest-websocket-mock
  await expect(server).toReceiveMessage(expectedMessage);
}

/**
 * Assert that the server has received specific messages
 *
 * @param expectedMessages - Array of expected messages
 *
 * @example
 * ```typescript
 * assertReceivedMessages([
 *   { type: 'subscribe', channel: 'daemon-state' },
 *   { type: 'query', method: 'getProfiles' }
 * ]);
 * ```
 */
export function assertReceivedMessages(expectedMessages: any[]): void {
  const server = getMockWebSocket();
  // Use the custom matcher provided by jest-websocket-mock
  expect(server).toHaveReceivedMessages(expectedMessages);
}

/**
 * Simulate WebSocket disconnection
 * Useful for testing offline/reconnection scenarios
 *
 * @example
 * ```typescript
 * simulateDisconnect();
 * // Wait for component to show "disconnected" state
 * await waitFor(() => expect(screen.getByText('Disconnected')).toBeInTheDocument());
 * ```
 */
export function simulateDisconnect(): void {
  const server = getMockWebSocket();
  server.close();
}

/**
 * Simulate WebSocket connection error
 * Useful for testing error handling
 *
 * @param error - Error to simulate
 *
 * @example
 * ```typescript
 * simulateError(new Error('Connection failed'));
 * ```
 */
export function simulateError(error?: Error): void {
  const server = getMockWebSocket();
  server.error(error);
}
