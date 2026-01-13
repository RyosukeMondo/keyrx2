import { expect, afterEach, beforeAll, beforeEach, afterAll } from 'vitest';
import { cleanup } from '@testing-library/react';
import * as matchers from '@testing-library/jest-dom/matchers';
import * as axeMatchers from 'vitest-axe/matchers';
import { server } from './mocks/server';
import { resetMockData } from './mocks/handlers';
import { setupMockWebSocket, cleanupMockWebSocket } from '../../tests/helpers/websocket';

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
 * WebSocket setup is now automatic for all tests (as of task 1.1).
 * Tests can still call setupMockWebSocket() manually if needed for custom configuration.
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
 * Setup WebSocket mock before each test
 *
 * This ensures all tests have a properly configured WebSocket mock,
 * preventing "assertIsWebSocket" errors from react-use-websocket.
 *
 * Tests can still override this by calling setupMockWebSocket() again
 * with custom configuration if needed.
 */
beforeEach(async () => {
  await setupMockWebSocket();
});

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
 * 3. Cleaning up WebSocket mock (automatic as of task 1.1)
 */
afterEach(() => {
  cleanup();
  // Reset HTTP mock data to prevent test pollution
  resetMockData();
  // Clean up WebSocket mock
  cleanupMockWebSocket();
});
