/**
 * Comprehensive WebSocket Mock for Testing
 *
 * This mock implements a full WebSocket interface compatible with:
 * - react-use-websocket library
 * - MSW (Mock Service Worker) WebSocket interception
 * - Test-controlled message injection and state management
 *
 * Features:
 * - Full EventTarget implementation
 * - Controllable connection states
 * - Message queue for async simulation
 * - Error injection for testing error handling
 * - Connection lifecycle hooks
 */

import type { ServerMessage, ClientMessage } from '../../src/types/rpc';

export type WebSocketMockState = 'connecting' | 'open' | 'closing' | 'closed';

export interface WebSocketMockOptions {
  /** Auto-connect on creation (default: true) */
  autoConnect?: boolean;
  /** Delay before onopen fires (default: 0ms) */
  connectionDelay?: number;
  /** Simulate connection error (default: false) */
  shouldError?: boolean;
  /** Error to throw when shouldError=true */
  errorMessage?: string;
}

/**
 * Comprehensive WebSocket mock for testing
 *
 * Compatible with react-use-websocket's expectations:
 * - Extends EventTarget for addEventListener/removeEventListener
 * - Implements all WebSocket constants and properties
 * - Provides test-controlled message injection
 */
export class MockWebSocket extends EventTarget implements WebSocket {
  // WebSocket constants
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSING = 2;
  static readonly CLOSED = 3;

  readonly CONNECTING = 0;
  readonly OPEN = 1;
  readonly CLOSING = 2;
  readonly CLOSED = 3;

  // WebSocket properties
  url: string;
  readyState: number = MockWebSocket.CONNECTING;
  protocol: string = '';
  extensions: string = '';
  bufferedAmount: number = 0;
  binaryType: BinaryType = 'blob';

  // Event handlers
  onopen: ((event: Event) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;

  // Test control properties
  private messageQueue: string[] = [];
  private options: Required<WebSocketMockOptions>;
  private openTimer: NodeJS.Timeout | null = null;
  private sentMessages: string[] = [];

  constructor(url: string, _protocols?: string | string[], options: WebSocketMockOptions = {}) {
    super();
    this.url = url;

    // Apply default options
    this.options = {
      autoConnect: options.autoConnect ?? true,
      connectionDelay: options.connectionDelay ?? 0,
      shouldError: options.shouldError ?? false,
      errorMessage: options.errorMessage ?? 'WebSocket connection failed',
    };

    // Auto-connect if enabled
    if (this.options.autoConnect) {
      this.simulateConnection();
    }
  }

  /**
   * Simulate opening the WebSocket connection
   */
  private simulateConnection(): void {
    this.openTimer = setTimeout(() => {
      if (this.options.shouldError) {
        this.simulateError(this.options.errorMessage);
        this.simulateClose(1006, 'Connection failed'); // 1006 = Abnormal Closure
        return;
      }

      this.readyState = MockWebSocket.OPEN;
      const openEvent = new Event('open');

      // Trigger both addEventListener and onopen handler
      this.dispatchEvent(openEvent);
      if (this.onopen) {
        this.onopen(openEvent);
      }

      // Send queued messages
      this.flushMessageQueue();
    }, this.options.connectionDelay);
  }

  /**
   * Simulate receiving a message from the server
   */
  simulateMessage(data: string | ServerMessage): void {
    if (this.readyState !== MockWebSocket.OPEN) {
      // Queue message if not yet connected
      const dataStr = typeof data === 'string' ? data : JSON.stringify(data);
      this.messageQueue.push(dataStr);
      return;
    }

    const dataStr = typeof data === 'string' ? data : JSON.stringify(data);
    const messageEvent = new MessageEvent('message', { data: dataStr });

    // Trigger both addEventListener and onmessage handler
    this.dispatchEvent(messageEvent);
    if (this.onmessage) {
      this.onmessage(messageEvent);
    }
  }

  /**
   * Simulate an error event
   */
  simulateError(message: string): void {
    const errorEvent = new Event('error');
    (errorEvent as any).message = message;

    this.dispatchEvent(errorEvent);
    if (this.onerror) {
      this.onerror(errorEvent);
    }
  }

  /**
   * Simulate closing the connection
   */
  simulateClose(code: number = 1000, reason: string = ''): void {
    if (this.readyState === MockWebSocket.CLOSED) {
      return;
    }

    this.readyState = MockWebSocket.CLOSED;
    const closeEvent = new CloseEvent('close', {
      code,
      reason,
      wasClean: code === 1000,
    });

    this.dispatchEvent(closeEvent);
    if (this.onclose) {
      this.onclose(closeEvent);
    }
  }

  /**
   * Send a message to the server
   */
  send(data: string | ArrayBuffer | Blob): void {
    if (this.readyState !== MockWebSocket.OPEN) {
      console.warn('[MockWebSocket] Cannot send message, not connected');
      return;
    }

    const dataStr = typeof data === 'string' ? data : '<binary>';
    this.sentMessages.push(dataStr);

    // Auto-respond to handshake messages
    if (typeof data === 'string') {
      try {
        const message: ClientMessage = JSON.parse(data);

        // Auto-respond to subscribe/unsubscribe
        if (message.type === 'subscribe' || message.type === 'unsubscribe') {
          // No response needed for subscriptions in the mock
          return;
        }

        // Auto-respond to queries/commands with mock response
        if (message.type === 'query' || message.type === 'command') {
          setTimeout(() => {
            this.simulateMessage({
              type: 'response',
              id: message.id,
              result: { success: true },
            } as ServerMessage);
          }, 10);
        }
      } catch (error) {
        // Not JSON, ignore
      }
    }
  }

  /**
   * Close the WebSocket connection
   */
  close(code: number = 1000, reason: string = ''): void {
    if (this.readyState === MockWebSocket.CLOSING || this.readyState === MockWebSocket.CLOSED) {
      return;
    }

    this.readyState = MockWebSocket.CLOSING;

    // Clear connection timer if still connecting
    if (this.openTimer) {
      clearTimeout(this.openTimer);
      this.openTimer = null;
    }

    // Simulate close event
    setTimeout(() => {
      this.simulateClose(code, reason);
    }, 0);
  }

  /**
   * Flush queued messages (sent after connection opens)
   */
  private flushMessageQueue(): void {
    while (this.messageQueue.length > 0) {
      const data = this.messageQueue.shift();
      if (data) {
        this.simulateMessage(data);
      }
    }
  }

  /**
   * Get all messages sent to the server (for test assertions)
   */
  getSentMessages(): string[] {
    return [...this.sentMessages];
  }

  /**
   * Get the last message sent to the server
   */
  getLastSentMessage(): string | undefined {
    return this.sentMessages[this.sentMessages.length - 1];
  }

  /**
   * Clear sent messages history
   */
  clearSentMessages(): void {
    this.sentMessages = [];
  }

  /**
   * Simulate a "Connected" handshake from the server
   */
  simulateConnectedHandshake(sessionId: string = 'test-session-123'): void {
    this.simulateMessage({
      type: 'connected',
      sessionId,
      timestamp: Date.now(),
    } as ServerMessage);
  }

  /**
   * Simulate a daemon state update
   */
  simulateDaemonState(state: Record<string, unknown>): void {
    this.simulateMessage({
      type: 'event',
      channel: 'daemon-state',
      data: state,
    } as ServerMessage);
  }

  /**
   * Simulate a latency stats update
   */
  simulateLatencyStats(stats: Record<string, unknown>): void {
    this.simulateMessage({
      type: 'event',
      channel: 'latency',
      data: stats,
    } as ServerMessage);
  }

  /**
   * Simulate an event record
   */
  simulateEvent(event: Record<string, unknown>): void {
    this.simulateMessage({
      type: 'event',
      channel: 'events',
      data: event,
    } as ServerMessage);
  }
}

/**
 * Global WebSocket mock instance for test control
 */
let currentMockWebSocket: MockWebSocket | null = null;

/**
 * Get the current mock WebSocket instance for test control
 */
export function getCurrentMockWebSocket(): MockWebSocket | null {
  return currentMockWebSocket;
}

/**
 * Create a mock WebSocket factory for testing
 *
 * This factory function replaces the global WebSocket constructor
 * during tests, allowing full control over WebSocket behavior.
 *
 * IMPORTANT: To make instances pass `instanceof WebSocket` checks,
 * we need to preserve the original WebSocket constructor and make
 * our mock instances inherit from it.
 */
export function createMockWebSocketFactory(
  defaultOptions: WebSocketMockOptions = {}
): typeof WebSocket {
  // Store reference to original WebSocket if it exists
  const OriginalWebSocket = typeof WebSocket !== 'undefined' ? WebSocket : null;

  const MockWebSocketFactory = class extends MockWebSocket {
    constructor(url: string, protocols?: string | string[]) {
      super(url, protocols, defaultOptions);
      currentMockWebSocket = this;

      // If there was an original WebSocket, make this instance's prototype chain include it
      // This makes `instanceof WebSocket` checks pass
      if (OriginalWebSocket) {
        Object.setPrototypeOf(this, OriginalWebSocket.prototype);
      }
    }
  };

  // Set the name for better debugging
  Object.defineProperty(MockWebSocketFactory, 'name', {
    value: 'WebSocket',
    writable: false,
  });

  // Copy static properties
  MockWebSocketFactory.CONNECTING = MockWebSocket.CONNECTING;
  MockWebSocketFactory.OPEN = MockWebSocket.OPEN;
  MockWebSocketFactory.CLOSING = MockWebSocket.CLOSING;
  MockWebSocketFactory.CLOSED = MockWebSocket.CLOSED;

  // If there was an original WebSocket, set the prototype
  if (OriginalWebSocket) {
    Object.setPrototypeOf(MockWebSocketFactory.prototype, OriginalWebSocket.prototype);
  }

  return MockWebSocketFactory as any;
}

/**
 * Reset the WebSocket mock state
 */
export function resetWebSocketMock(): void {
  if (currentMockWebSocket) {
    currentMockWebSocket.close();
    currentMockWebSocket = null;
  }
}
