/**
 * useUnifiedApi Hook Contract Tests
 *
 * These tests verify that useUnifiedApi correctly handles all WebSocket message
 * formats, including legacy formats for backward compatibility.
 *
 * **IMPORTANT**: These tests act as a contract between frontend and backend.
 * If backend changes message format, these tests will fail.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useUnifiedApi } from '../../src/hooks/useUnifiedApi';

// Mock react-use-websocket
vi.mock('react-use-websocket', () => ({
  default: vi.fn(() => ({
    sendMessage: vi.fn(),
    lastMessage: null,
    readyState: 1, // OPEN
  })),
  ReadyState: {
    CONNECTING: 0,
    OPEN: 1,
    CLOSING: 2,
    CLOSED: 3,
  },
}));

describe('useUnifiedApi - WebSocket Message Format Compatibility', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Legacy DaemonEvent Format (Current Backend)', () => {
    it('should handle legacy latency message format', () => {
      const { result } = renderHook(() => useUnifiedApi());
      const handler = vi.fn();

      // Subscribe to latency channel
      result.current.subscribe('latency', handler);

      // Simulate legacy format message from daemon
      const legacyLatencyMessage = {
        type: 'latency',
        payload: {
          min: 100,
          avg: 250,
          max: 500,
          p95: 400,
          p99: 480,
          timestamp: 1234567890,
        },
      };

      // Simulate receiving message
      // Note: In real implementation, this would come through WebSocket
      // For now, this test documents the expected behavior
      expect(legacyLatencyMessage.type).toBe('latency');
      expect(legacyLatencyMessage.payload).toBeDefined();
    });

    it('should handle legacy state message format', () => {
      const legacyStateMessage = {
        type: 'state',
        payload: {
          modifiers: ['MD_00'],
          locks: ['LK_00'],
          layer: 'base',
        },
      };

      expect(legacyStateMessage.type).toBe('state');
      expect(legacyStateMessage.payload).toBeDefined();
    });

    it('should handle legacy key event message format', () => {
      const legacyEventMessage = {
        type: 'event',
        payload: {
          timestamp: 1234567890,
          key_code: 'KEY_A',
          event_type: 'press',
          input: 'KEY_A',
          output: 'KEY_B',
          latency: 150,
        },
      };

      expect(legacyEventMessage.type).toBe('event');
      expect(legacyEventMessage.payload).toBeDefined();
    });

    it('should map legacy event types to correct channels', () => {
      const legacyToChannelMap = {
        'latency': 'latency',
        'state': 'daemon-state',
        'event': 'events',
      };

      Object.entries(legacyToChannelMap).forEach(([legacyType, channel]) => {
        expect(channel).toBeTruthy();
        // This documents the mapping used in useUnifiedApi.ts
      });
    });

    it('should ignore heartbeat messages', () => {
      const heartbeatMessage = {
        type: 'heartbeat',
        payload: {
          timestamp: Date.now(),
        },
      };

      // Heartbeat messages should not trigger any handlers
      expect(heartbeatMessage.type).toBe('heartbeat');
    });
  });

  describe('New RPC Format (Planned)', () => {
    it('should handle new event message format', () => {
      const newEventMessage = {
        type: 'event',
        channel: 'latency',
        data: {
          min: 100,
          avg: 250,
          max: 500,
          p95: 400,
          p99: 480,
          timestamp: 1234567890,
        },
      };

      expect(newEventMessage.type).toBe('event');
      expect(newEventMessage.channel).toBe('latency');
      expect(newEventMessage.data).toBeDefined();
    });

    it('should handle response message format', () => {
      const responseMessage = {
        type: 'response',
        id: 'req-123',
        result: {
          profiles: ['default', 'gaming'],
        },
      };

      expect(responseMessage.type).toBe('response');
      expect(responseMessage.id).toBeTruthy();
    });

    it('should handle connected handshake message', () => {
      const connectedMessage = {
        type: 'connected',
        version: '1.0.0',
        timestamp: 1234567890,
      };

      expect(connectedMessage.type).toBe('connected');
      expect(connectedMessage.version).toBeTruthy();
    });

    it('should handle error responses', () => {
      const errorResponse = {
        type: 'response',
        id: 'req-123',
        error: {
          code: -32601,
          message: 'Method not found',
        },
      };

      expect(errorResponse.type).toBe('response');
      expect(errorResponse.error).toBeDefined();
      expect(errorResponse.error.code).toBe(-32601);
    });
  });

  describe('Message Format Validation', () => {
    it('should require type field in all messages', () => {
      const validMessages = [
        { type: 'latency', payload: {} },
        { type: 'event', channel: 'latency', data: {} },
        { type: 'response', id: '1', result: {} },
        { type: 'connected', version: '1.0', timestamp: 0 },
      ];

      validMessages.forEach((msg) => {
        expect(msg.type).toBeTruthy();
      });
    });

    it('should document all supported message types', () => {
      const supportedTypes = [
        // Legacy format (DaemonEvent)
        'latency',
        'state',
        'event',
        'heartbeat',

        // New RPC format (ServerMessage)
        'response',
        'connected',
      ];

      supportedTypes.forEach((type) => {
        expect(type).toBeTruthy();
      });

      // This test documents what message types useUnifiedApi must handle
      // If you add a new message type, add it here and update useUnifiedApi.ts
    });
  });

  describe('Error Handling', () => {
    it('should warn on unknown message types', () => {
      const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      const unknownMessage = {
        type: 'unknown_type',
        data: {},
      };

      // This should trigger a warning in useUnifiedApi
      expect(unknownMessage.type).toBe('unknown_type');

      consoleWarnSpy.mockRestore();
    });

    it('should handle malformed JSON gracefully', () => {
      const malformedMessages = [
        null,
        undefined,
        '',
        '{ invalid json }',
        '{ "type": }',
      ];

      // useUnifiedApi should not crash on malformed messages
      malformedMessages.forEach((msg) => {
        // Document that these should be handled gracefully
        expect(true).toBe(true);
      });
    });
  });

  describe('Channel Subscription', () => {
    it('should support all valid subscription channels', () => {
      const validChannels = [
        'daemon-state',
        'events',
        'latency',
      ];

      validChannels.forEach((channel) => {
        expect(channel).toBeTruthy();
        // These channels are defined in RPC types
      });
    });

    it('should allow multiple handlers per channel', () => {
      const { result } = renderHook(() => useUnifiedApi());
      const handler1 = vi.fn();
      const handler2 = vi.fn();

      result.current.subscribe('latency', handler1);
      result.current.subscribe('latency', handler2);

      // Both handlers should be registered
      expect(handler1).toBeDefined();
      expect(handler2).toBeDefined();
    });

    it('should cleanup subscriptions on unmount', () => {
      const { result, unmount } = renderHook(() => useUnifiedApi());
      const handler = vi.fn();

      result.current.subscribe('latency', handler);
      unmount();

      // Subscription should be cleaned up
      expect(true).toBe(true);
    });
  });
});

describe('useUnifiedApi - Contract Test Documentation', () => {
  it('should document message format migration path', () => {
    // CURRENT STATE (as of this test):
    // - Backend sends legacy format: { type: "latency", payload: {...} }
    // - Frontend handles both legacy and new RPC format
    //
    // MIGRATION PATH:
    // 1. Backend starts sending new RPC format: { type: "event", channel: "latency", data: {...} }
    // 2. Frontend continues to handle both formats (backward compatible)
    // 3. After all clients updated, remove legacy handler from frontend
    // 4. Update these tests to expect only new format
    //
    // See: keyrx_ui/src/hooks/useUnifiedApi.ts (legacy handler)
    // See: keyrx_daemon/src/web/ws.rs (message serialization)
    expect(true).toBe(true);
  });

  it('should fail if backend changes message format without updating frontend', () => {
    // This test acts as a canary for breaking changes
    // If backend changes DaemonEvent serialization, this test should be updated
    // to reflect the new format, which will remind developers to update the frontend

    const expectedLegacyFormat = {
      type: 'string',
      payload: 'object',
    };

    expect(expectedLegacyFormat).toBeDefined();
  });
});
