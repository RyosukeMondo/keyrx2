import { expect, afterEach, beforeAll, afterAll, vi } from 'vitest';
import { cleanup } from '@testing-library/react';
import * as matchers from '@testing-library/jest-dom/matchers';
import * as axeMatchers from 'vitest-axe/matchers';
import { server } from './mocks/server';
import { resetMockData } from './mocks/handlers';

// Mock react-use-websocket's assertIsWebSocket to skip instanceof check
// This allows jest-websocket-mock's WebSocket to pass the check
vi.mock('react-use-websocket/src/lib/util', async () => {
  const actual = await vi.importActual<typeof import('react-use-websocket/src/lib/util')>('react-use-websocket/src/lib/util');
  return {
    ...actual,
    assertIsWebSocket: vi.fn(), // No-op: skip instanceof check
  };
});

// Extend Vitest's expect with jest-dom matchers
expect.extend(matchers);

// Extend Vitest's expect with axe matchers
expect.extend(axeMatchers);

// Mock window.matchMedia for animation tests
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {}, // deprecated
    removeListener: () => {}, // deprecated
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => true,
  }),
});

// Mock scrollIntoView for jsdom
Element.prototype.scrollIntoView = function() {
  // No-op implementation for tests
};

// Mock ResizeObserver for recharts/responsive components
global.ResizeObserver = class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
};

// WebSocket mocking is now handled by jest-websocket-mock
// Tests should use setupMockWebSocket() from tests/helpers/websocket.ts
// Make global.WebSocket writable so jest-websocket-mock can replace it
Object.defineProperty(global, 'WebSocket', {
  writable: true,
  configurable: true,
  value: global.WebSocket,
});

// MSW server setup
beforeAll(() =>
  server.listen({
    // Allow WebSocket connections to use our mock instead of MSW interception
    onUnhandledRequest(request) {
      // Bypass WebSocket upgrade requests - let our mock handle them
      if (request.url.startsWith('ws:') || request.url.startsWith('wss:')) {
        return;
      }

      // Error on other unhandled HTTP requests
      console.error(`[MSW] Unhandled ${request.method} request to ${request.url}`);
      throw new Error(
        `Unhandled ${request.method} request to ${request.url}. ` +
          `Add a handler for this endpoint in src/test/mocks/handlers.ts`
      );
    },
  })
);
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

// Cleanup after each test
afterEach(() => {
  cleanup();
  // Reset MSW mock data to prevent test pollution
  resetMockData();
  // Note: WebSocket cleanup is handled by individual tests using cleanupMockWebSocket()
});
