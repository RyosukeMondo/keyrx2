/**
 * WebSocket client for KeyRx daemon real-time events
 *
 * Provides:
 * - Connection management with auto-reconnect
 * - Channel subscription (devices, profiles, metrics, state)
 * - Event filtering and waiting with timeout
 * - Graceful disconnect and cleanup
 */

import WebSocket from 'ws';

/**
 * WebSocket client configuration
 */
export interface WebSocketClientConfig {
  url: string;
  reconnect?: boolean;
  reconnectDelayMs?: number;
  maxReconnectAttempts?: number;
  pingIntervalMs?: number;
}

/**
 * WebSocket message types
 */
export interface SubscribeMessage {
  type: 'subscribe';
  channel: string;
}

export interface UnsubscribeMessage {
  type: 'unsubscribe';
  channel: string;
}

export interface SubscriptionAckMessage {
  type: 'subscription_ack';
  channel: string;
  success: boolean;
  message?: string;
}

export interface EventMessage {
  type: 'event';
  channel: string;
  event: string;
  data: unknown;
  timestamp: string;
}

export type WebSocketMessage =
  | SubscribeMessage
  | UnsubscribeMessage
  | SubscriptionAckMessage
  | EventMessage;

/**
 * Connection states
 */
export enum ConnectionState {
  DISCONNECTED = 'disconnected',
  CONNECTING = 'connecting',
  CONNECTED = 'connected',
  RECONNECTING = 'reconnecting',
  CLOSED = 'closed',
}

/**
 * Event predicate for filtering events
 */
export type EventPredicate = (event: EventMessage) => boolean;

/**
 * WebSocket client error types
 */
export class WebSocketClientError extends Error {
  constructor(message: string, public readonly cause?: Error) {
    super(message);
    this.name = 'WebSocketClientError';
  }
}

export class ConnectionError extends WebSocketClientError {
  constructor(message: string, cause?: Error) {
    super(message, cause);
    this.name = 'ConnectionError';
  }
}

export class TimeoutError extends WebSocketClientError {
  constructor(message: string) {
    super(message);
    this.name = 'TimeoutError';
  }
}

export class SubscriptionError extends WebSocketClientError {
  constructor(
    message: string,
    public readonly channel: string
  ) {
    super(message);
    this.name = 'SubscriptionError';
  }
}

/**
 * WebSocket client for real-time communication with daemon
 */
export class WebSocketClient {
  private ws: WebSocket | null = null;
  private state: ConnectionState = ConnectionState.DISCONNECTED;
  private readonly config: Required<WebSocketClientConfig>;
  private reconnectAttempts = 0;
  private reconnectTimer: NodeJS.Timeout | null = null;
  private pingTimer: NodeJS.Timeout | null = null;
  private readonly subscriptions = new Set<string>();
  private readonly eventHandlers: Array<(event: EventMessage) => void> = [];
  private readonly messageQueue: EventMessage[] = [];

  constructor(config: WebSocketClientConfig) {
    this.config = {
      url: config.url,
      reconnect: config.reconnect ?? false,
      reconnectDelayMs: config.reconnectDelayMs ?? 1000,
      maxReconnectAttempts: config.maxReconnectAttempts ?? 5,
      pingIntervalMs: config.pingIntervalMs ?? 30000,
    };
  }

  /**
   * Get current connection state
   */
  getState(): ConnectionState {
    return this.state;
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.state === ConnectionState.CONNECTED;
  }

  /**
   * Get active subscriptions
   */
  getSubscriptions(): Set<string> {
    return new Set(this.subscriptions);
  }

  /**
   * Connect to WebSocket server
   */
  async connect(timeoutMs = 5000): Promise<void> {
    if (this.state === ConnectionState.CONNECTED) {
      return; // Already connected
    }

    if (this.state === ConnectionState.CONNECTING) {
      throw new ConnectionError('Connection already in progress');
    }

    this.state = ConnectionState.CONNECTING;

    return new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        this.cleanup();
        reject(new TimeoutError(`Connection timeout after ${timeoutMs}ms`));
      }, timeoutMs);

      try {
        this.ws = new WebSocket(this.config.url);

        this.ws.on('open', () => {
          clearTimeout(timeoutId);
          this.state = ConnectionState.CONNECTED;
          this.reconnectAttempts = 0;
          this.startPingTimer();
          resolve();
        });

        this.ws.on('message', (data: WebSocket.Data) => {
          this.handleMessage(data);
        });

        this.ws.on('error', (error: Error) => {
          clearTimeout(timeoutId);
          this.handleError(error);
          if (this.state === ConnectionState.CONNECTING) {
            reject(new ConnectionError('Connection failed', error));
          }
        });

        this.ws.on('close', () => {
          clearTimeout(timeoutId);
          this.handleClose();
        });
      } catch (error) {
        clearTimeout(timeoutId);
        this.state = ConnectionState.DISCONNECTED;
        reject(
          new ConnectionError(
            'Failed to create WebSocket',
            error as Error
          )
        );
      }
    });
  }

  /**
   * Disconnect from WebSocket server
   */
  disconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

    this.cleanup();
    this.state = ConnectionState.CLOSED;
  }

  /**
   * Subscribe to a channel
   */
  async subscribe(channel: string, timeoutMs = 5000): Promise<void> {
    if (!this.isConnected()) {
      throw new SubscriptionError(
        'Cannot subscribe: not connected',
        channel
      );
    }

    if (this.subscriptions.has(channel)) {
      return; // Already subscribed
    }

    return new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        reject(
          new TimeoutError(
            `Subscription timeout for channel: ${channel}`
          )
        );
      }, timeoutMs);

      // Handler for subscription acknowledgment
      const ackHandler = (event: EventMessage) => {
        const msg = event as unknown as SubscriptionAckMessage;
        if (
          msg.type === 'subscription_ack' &&
          msg.channel === channel
        ) {
          clearTimeout(timeoutId);
          this.removeEventHandler(ackHandler);

          if (msg.success) {
            this.subscriptions.add(channel);
            resolve();
          } else {
            reject(
              new SubscriptionError(
                msg.message || 'Subscription failed',
                channel
              )
            );
          }
        }
      };

      this.addEventHandler(ackHandler);

      // Send subscription message
      const subscribeMsg: SubscribeMessage = {
        type: 'subscribe',
        channel,
      };
      this.send(subscribeMsg);
    });
  }

  /**
   * Unsubscribe from a channel
   */
  unsubscribe(channel: string): void {
    if (!this.subscriptions.has(channel)) {
      return; // Not subscribed
    }

    const unsubscribeMsg: UnsubscribeMessage = {
      type: 'unsubscribe',
      channel,
    };
    this.send(unsubscribeMsg);
    this.subscriptions.delete(channel);
  }

  /**
   * Wait for an event matching the predicate
   */
  async waitForEvent(
    predicate: EventPredicate,
    timeoutMs = 5000
  ): Promise<EventMessage> {
    // Check if event already in queue
    const existingEvent = this.messageQueue.find(predicate);
    if (existingEvent) {
      // Remove from queue
      const index = this.messageQueue.indexOf(existingEvent);
      this.messageQueue.splice(index, 1);
      return existingEvent;
    }

    return new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        this.removeEventHandler(handler);
        reject(new TimeoutError('Event wait timeout'));
      }, timeoutMs);

      const handler = (event: EventMessage) => {
        if (predicate(event)) {
          clearTimeout(timeoutId);
          this.removeEventHandler(handler);
          resolve(event);
        }
      };

      this.addEventHandler(handler);
    });
  }

  /**
   * Get all received events (for testing/debugging)
   */
  getReceivedEvents(): EventMessage[] {
    return [...this.messageQueue];
  }

  /**
   * Clear event queue
   */
  clearEventQueue(): void {
    this.messageQueue.length = 0;
  }

  /**
   * Send message to server
   */
  private send(message: WebSocketMessage): void {
    if (!this.ws || this.state !== ConnectionState.CONNECTED) {
      throw new WebSocketClientError('Cannot send: not connected');
    }

    this.ws.send(JSON.stringify(message));
  }

  /**
   * Handle incoming message
   */
  private handleMessage(data: WebSocket.Data): void {
    try {
      const message = JSON.parse(data.toString()) as WebSocketMessage;

      // Only queue and dispatch event messages
      if (message.type === 'event') {
        const eventMsg = message as EventMessage;
        this.messageQueue.push(eventMsg);

        // Dispatch to handlers
        for (const handler of this.eventHandlers) {
          try {
            handler(eventMsg);
          } catch (error) {
            console.error('Event handler error:', error);
          }
        }
      } else {
        // Non-event messages (like subscription_ack) also get dispatched
        // but not queued
        for (const handler of this.eventHandlers) {
          try {
            handler(message as unknown as EventMessage);
          } catch (error) {
            console.error('Event handler error:', error);
          }
        }
      }
    } catch (error) {
      console.error('Failed to parse WebSocket message:', error);
    }
  }

  /**
   * Handle WebSocket error
   */
  private handleError(error: Error): void {
    console.error('WebSocket error:', error);
  }

  /**
   * Handle WebSocket close
   */
  private handleClose(): void {
    this.stopPingTimer();

    if (
      this.config.reconnect &&
      this.state !== ConnectionState.CLOSED &&
      this.reconnectAttempts < this.config.maxReconnectAttempts
    ) {
      this.state = ConnectionState.RECONNECTING;
      this.reconnectAttempts++;

      const delay =
        this.config.reconnectDelayMs *
        Math.pow(2, this.reconnectAttempts - 1);

      this.reconnectTimer = setTimeout(() => {
        this.connect().catch((error) => {
          console.error('Reconnection failed:', error);
          this.handleClose(); // Try again or give up
        });
      }, delay);
    } else {
      this.state = ConnectionState.DISCONNECTED;
      this.cleanup();
    }
  }

  /**
   * Start ping timer to keep connection alive
   */
  private startPingTimer(): void {
    this.pingTimer = setInterval(() => {
      if (this.ws && this.state === ConnectionState.CONNECTED) {
        this.ws.ping();
      }
    }, this.config.pingIntervalMs);
  }

  /**
   * Stop ping timer
   */
  private stopPingTimer(): void {
    if (this.pingTimer) {
      clearInterval(this.pingTimer);
      this.pingTimer = null;
    }
  }

  /**
   * Add event handler
   */
  private addEventHandler(
    handler: (event: EventMessage) => void
  ): void {
    this.eventHandlers.push(handler);
  }

  /**
   * Remove event handler
   */
  private removeEventHandler(
    handler: (event: EventMessage) => void
  ): void {
    const index = this.eventHandlers.indexOf(handler);
    if (index !== -1) {
      this.eventHandlers.splice(index, 1);
    }
  }

  /**
   * Cleanup internal state
   */
  private cleanup(): void {
    this.stopPingTimer();
    this.subscriptions.clear();
    this.eventHandlers.length = 0;
    this.messageQueue.length = 0;
  }
}

/**
 * Create WebSocket client instance
 *
 * @example
 * const client = createWebSocketClient({ url: 'ws://localhost:9867/ws' });
 * await client.connect();
 * await client.subscribe('devices');
 * const event = await client.waitForEvent(e => e.channel === 'devices');
 * client.disconnect();
 */
export function createWebSocketClient(
  config: WebSocketClientConfig
): WebSocketClient {
  return new WebSocketClient(config);
}
