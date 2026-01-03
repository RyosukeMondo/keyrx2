/**
 * Manual mock for react-use-websocket/src/lib/util
 * Bypasses the assertIsWebSocket instanceof check to allow our WebSocket mock
 */

// Re-export everything from the actual module
export * from 'react-use-websocket/src/lib/util';

// Override assertIsWebSocket to skip the instanceof check
export function assertIsWebSocket(): void {
  // No-op: skip instanceof check for our WebSocket mock in tests
}
