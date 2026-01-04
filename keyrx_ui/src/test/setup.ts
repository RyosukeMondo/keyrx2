import { expect, afterEach, beforeAll, afterAll } from 'vitest';
import { cleanup } from '@testing-library/react';
import * as matchers from '@testing-library/jest-dom/matchers';
import * as axeMatchers from 'vitest-axe/matchers';
import { server } from './mocks/server';
import { resetMockData } from './mocks/handlers';

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

/**
 * MSW server lifecycle hooks
 *
 * HYBRID MOCKING APPROACH:
 * - MSW handles HTTP REST API mocking (this server)
 * - jest-websocket-mock handles WebSocket mocking (tests/helpers/websocket.ts)
 *
 * This approach uses the best tool for each job:
 * - MSW: Proven excellent for HTTP mocking
 * - jest-websocket-mock: Better react-use-websocket compatibility
 *
 * WebSocket setup is handled per-test via setupMockWebSocket() from
 * tests/helpers/websocket.ts - not automatic like HTTP mocking.
 */
beforeAll(() =>
  server.listen({
    onUnhandledRequest(request) {
      // Skip WebSocket requests - handled by jest-websocket-mock
      if (request.url.startsWith('ws:') || request.url.startsWith('wss:')) {
        return;
      }

      // Log warning for unhandled HTTP requests
      console.error(`[MSW] Unhandled ${request.method} request to ${request.url}`);
      throw new Error(
        `Unhandled ${request.method} request to ${request.url}. ` +
          `Add a handler for this endpoint in src/test/mocks/handlers.ts`
      );
    },
  })
);

/**
 * Reset handlers between tests to prevent test pollution
 */
afterEach(() => {
  server.resetHandlers();
});

/**
 * Close MSW server after all tests complete
 */
afterAll(() => {
  server.close();
});

/**
 * Cleanup after each test
 *
 * This ensures test isolation by:
 * 1. Cleaning up React components
 * 2. Resetting HTTP mock data
 *
 * Note: WebSocket cleanup is handled by cleanupMockWebSocket() in individual
 * tests that use WebSocket mocking (see tests/helpers/websocket.ts).
 */
afterEach(() => {
  cleanup();
  // Reset HTTP mock data to prevent test pollution
  resetMockData();
});
