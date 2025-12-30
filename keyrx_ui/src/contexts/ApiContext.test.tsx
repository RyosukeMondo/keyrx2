/**
 * ApiContext Unit Tests
 *
 * Tests for API context provider and useApi hook.
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ApiProvider, useApi } from './ApiContext';

/**
 * Test component that uses the useApi hook
 */
function TestComponent() {
  const { apiBaseUrl, wsBaseUrl } = useApi();
  return (
    <div>
      <div data-testid="api-url">{apiBaseUrl}</div>
      <div data-testid="ws-url">{wsBaseUrl}</div>
    </div>
  );
}

describe('ApiContext', () => {
  describe('ApiProvider', () => {
    it('provides default API base URL', () => {
      render(
        <ApiProvider>
          <TestComponent />
        </ApiProvider>
      );

      expect(screen.getByTestId('api-url').textContent).toBe('http://localhost:3030');
    });

    it('provides default WebSocket base URL', () => {
      render(
        <ApiProvider>
          <TestComponent />
        </ApiProvider>
      );

      expect(screen.getByTestId('ws-url').textContent).toBe('ws://localhost:9867');
    });

    it('accepts custom API base URL prop', () => {
      render(
        <ApiProvider apiBaseUrl="http://custom-api:8080">
          <TestComponent />
        </ApiProvider>
      );

      expect(screen.getByTestId('api-url').textContent).toBe('http://custom-api:8080');
    });

    it('accepts custom WebSocket base URL prop', () => {
      render(
        <ApiProvider wsBaseUrl="ws://custom-ws:9999">
          <TestComponent />
        </ApiProvider>
      );

      expect(screen.getByTestId('ws-url').textContent).toBe('ws://custom-ws:9999');
    });

    it('accepts both custom URLs', () => {
      render(
        <ApiProvider apiBaseUrl="http://test-api:3000" wsBaseUrl="ws://test-ws:9000">
          <TestComponent />
        </ApiProvider>
      );

      expect(screen.getByTestId('api-url').textContent).toBe('http://test-api:3000');
      expect(screen.getByTestId('ws-url').textContent).toBe('ws://test-ws:9000');
    });

    it('renders children correctly', () => {
      render(
        <ApiProvider>
          <div data-testid="child">Child Component</div>
        </ApiProvider>
      );

      expect(screen.getByTestId('child')).toBeInTheDocument();
      expect(screen.getByTestId('child').textContent).toBe('Child Component');
    });
  });

  describe('useApi hook', () => {
    it('returns API context value', () => {
      render(
        <ApiProvider>
          <TestComponent />
        </ApiProvider>
      );

      const apiUrl = screen.getByTestId('api-url');
      const wsUrl = screen.getByTestId('ws-url');

      expect(apiUrl).toBeInTheDocument();
      expect(wsUrl).toBeInTheDocument();
    });

    it('throws error when used outside ApiProvider', () => {
      // Suppress console.error for this test
      const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});

      expect(() => {
        render(<TestComponent />);
      }).toThrow('useApi must be used within an ApiProvider');

      consoleError.mockRestore();
    });

    it('provides apiBaseUrl property', () => {
      render(
        <ApiProvider apiBaseUrl="http://example.com">
          <TestComponent />
        </ApiProvider>
      );

      expect(screen.getByTestId('api-url').textContent).toBe('http://example.com');
    });

    it('provides wsBaseUrl property', () => {
      render(
        <ApiProvider wsBaseUrl="ws://example.com">
          <TestComponent />
        </ApiProvider>
      );

      expect(screen.getByTestId('ws-url').textContent).toBe('ws://example.com');
    });
  });

  describe('Environment variable support', () => {
    it('uses defaults when environment variables are not set', () => {
      // When no env vars are set, defaults are used
      render(
        <ApiProvider>
          <TestComponent />
        </ApiProvider>
      );

      // Verify defaults are used
      expect(screen.getByTestId('api-url').textContent).toBe('http://localhost:3030');
      expect(screen.getByTestId('ws-url').textContent).toBe('ws://localhost:9867');
    });

    it('props override any defaults or environment variables', () => {
      render(
        <ApiProvider apiBaseUrl="http://prop-api:6000" wsBaseUrl="ws://prop-ws:7000">
          <TestComponent />
        </ApiProvider>
      );

      // Props should always take precedence
      expect(screen.getByTestId('api-url').textContent).toBe('http://prop-api:6000');
      expect(screen.getByTestId('ws-url').textContent).toBe('ws://prop-ws:7000');
    });
  });

  describe('Multiple components using useApi', () => {
    function ComponentA() {
      const { apiBaseUrl } = useApi();
      return <div data-testid="component-a">{apiBaseUrl}</div>;
    }

    function ComponentB() {
      const { wsBaseUrl } = useApi();
      return <div data-testid="component-b">{wsBaseUrl}</div>;
    }

    it('provides same context to all child components', () => {
      render(
        <ApiProvider apiBaseUrl="http://shared:3030" wsBaseUrl="ws://shared:9867">
          <ComponentA />
          <ComponentB />
        </ApiProvider>
      );

      expect(screen.getByTestId('component-a').textContent).toBe('http://shared:3030');
      expect(screen.getByTestId('component-b').textContent).toBe('ws://shared:9867');
    });
  });

  describe('Nested providers', () => {
    it('inner provider overrides outer provider', () => {
      render(
        <ApiProvider apiBaseUrl="http://outer:3030">
          <ApiProvider apiBaseUrl="http://inner:4040">
            <TestComponent />
          </ApiProvider>
        </ApiProvider>
      );

      expect(screen.getByTestId('api-url').textContent).toBe('http://inner:4040');
    });
  });
});
