/**
 * WebSocket Testing Infrastructure
 *
 * Uses jest-websocket-mock for robust WebSocket testing.
 * This library provides a mock WebSocket server that:
 * - Automatically integrates with @testing-library/react's act()
 * - Provides custom matchers for assertions
 * - Handles message serialization/deserialization
 * - Works seamlessly with Vitest
 */

import WS from 'jest-websocket-mock';

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
    WS.clean();
    mockServer = null;
  }

  // Create new server with JSON protocol enabled by default
  mockServer = new WS(url, { jsonProtocol: true, ...options });

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
