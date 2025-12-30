/**
 * Unit tests for DashboardPage component
 *
 * Tests dashboard layout, child component integration, WebSocket connection states,
 * and overall page behavior.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, within } from '@testing-library/react';
import { DashboardPage } from './DashboardPage';
import { useDashboardStore } from '../store/dashboardStore';
import { ReadyState } from '../hooks/useDaemonWebSocket';

// Mock the child components to isolate DashboardPage testing
vi.mock('./StateIndicatorPanel', () => ({
  StateIndicatorPanel: () => <div data-testid="state-indicator-panel">State Indicator Panel</div>,
}));

vi.mock('./MetricsChart', () => ({
  MetricsChart: () => <div data-testid="metrics-chart">Metrics Chart</div>,
}));

vi.mock('./DashboardEventTimeline', () => ({
  DashboardEventTimeline: ({ height }: { height: number }) => (
    <div data-testid="event-timeline" data-height={height}>Event Timeline</div>
  ),
}));

// Mock the useDaemonWebSocket hook
vi.mock('../hooks/useDaemonWebSocket', () => ({
  useDaemonWebSocket: vi.fn(),
  ReadyState: {
    CONNECTING: 0,
    OPEN: 1,
    CLOSING: 2,
    CLOSED: 3,
    UNINSTANTIATED: -1,
  },
}));

import { useDaemonWebSocket } from '../hooks/useDaemonWebSocket';

describe('DashboardPage', () => {
  beforeEach(() => {
    // Reset the store before each test
    useDashboardStore.getState().reset();

    // Default mock: WebSocket is disconnected
    vi.mocked(useDaemonWebSocket).mockReturnValue({
      readyState: ReadyState.CLOSED,
      sendMessage: vi.fn(),
      lastMessage: null,
      isConnected: false,
      isConnecting: false,
      isDisconnected: true,
    });
  });

  describe('Page Layout', () => {
    it('should render the dashboard page with all child components', () => {
      render(<DashboardPage />);

      expect(screen.getByTestId('state-indicator-panel')).toBeInTheDocument();
      expect(screen.getByTestId('metrics-chart')).toBeInTheDocument();
      expect(screen.getByTestId('event-timeline')).toBeInTheDocument();
    });

    it('should render connection status banner', () => {
      render(<DashboardPage />);

      const banner = screen.getByText(/disconnected/i).closest('.connection-banner');
      expect(banner).toBeInTheDocument();
      expect(banner).toHaveClass('connection-disconnected');
    });

    it('should render dashboard grid layout', () => {
      const { container } = render(<DashboardPage />);

      const grid = container.querySelector('.dashboard-grid');
      expect(grid).toBeInTheDocument();

      const panels = container.querySelectorAll('.panel');
      expect(panels).toHaveLength(3); // state, metrics, events
    });

    it('should render state panel with correct heading', () => {
      render(<DashboardPage />);

      const statePanel = screen.getByTestId('state-indicator-panel').closest('.panel');
      const heading = within(statePanel!).getByRole('heading', { name: /daemon state/i });
      expect(heading).toBeInTheDocument();
      expect(heading.tagName).toBe('H2');
    });

    it('should render metrics panel with correct heading', () => {
      render(<DashboardPage />);

      const metricsPanel = screen.getByTestId('metrics-chart').closest('.panel');
      const heading = within(metricsPanel!).getByRole('heading', { name: /latency metrics/i });
      expect(heading).toBeInTheDocument();
      expect(heading.tagName).toBe('H2');
    });

    it('should pass correct height prop to DashboardEventTimeline', () => {
      render(<DashboardPage />);

      const timeline = screen.getByTestId('event-timeline');
      expect(timeline).toHaveAttribute('data-height', '500');
    });
  });

  describe('Connection Status Display', () => {
    it('should display connecting status when WebSocket is connecting', () => {
      useDashboardStore.getState().setConnectionStatus('connecting');
      vi.mocked(useDaemonWebSocket).mockReturnValue({
        readyState: ReadyState.CONNECTING,
        sendMessage: vi.fn(),
        lastMessage: null,
        isConnected: false,
        isConnecting: true,
        isDisconnected: false,
      });

      render(<DashboardPage />);

      expect(screen.getByText(/connecting to daemon/i)).toBeInTheDocument();

      const statusIndicator = screen.getByText(/connecting to daemon/i)
        .previousElementSibling;
      expect(statusIndicator).toHaveClass('status-indicator', 'connecting');
    });

    it('should display connected status when WebSocket is open', () => {
      useDashboardStore.getState().setConnectionStatus('connected');
      vi.mocked(useDaemonWebSocket).mockReturnValue({
        readyState: ReadyState.OPEN,
        sendMessage: vi.fn(),
        lastMessage: null,
        isConnected: true,
        isConnecting: false,
        isDisconnected: false,
      });

      render(<DashboardPage />);

      expect(screen.getByText(/connected to daemon/i)).toBeInTheDocument();

      const statusIndicator = screen.getByText(/connected to daemon/i)
        .previousElementSibling;
      expect(statusIndicator).toHaveClass('status-indicator', 'connected');
    });

    it('should display disconnected status when WebSocket is closed', () => {
      useDashboardStore.getState().setConnectionStatus('disconnected');
      vi.mocked(useDaemonWebSocket).mockReturnValue({
        readyState: ReadyState.CLOSED,
        sendMessage: vi.fn(),
        lastMessage: null,
        isConnected: false,
        isConnecting: false,
        isDisconnected: true,
      });

      render(<DashboardPage />);

      expect(screen.getByText(/disconnected.*attempting to reconnect/i)).toBeInTheDocument();

      const statusIndicator = screen.getByText(/disconnected.*attempting to reconnect/i)
        .previousElementSibling;
      expect(statusIndicator).toHaveClass('status-indicator', 'disconnected');
    });

    it('should apply correct CSS class to connection banner based on status', () => {
      const testCases: Array<['connecting' | 'connected' | 'disconnected', string]> = [
        ['connecting', 'connection-connecting'],
        ['connected', 'connection-connected'],
        ['disconnected', 'connection-disconnected'],
      ];

      testCases.forEach(([status, expectedClass]) => {
        useDashboardStore.getState().setConnectionStatus(status);

        const { container, unmount } = render(<DashboardPage />);

        const banner = container.querySelector('.connection-banner');
        expect(banner).toHaveClass(expectedClass);

        unmount();
      });
    });
  });

  describe('WebSocket Integration', () => {
    it('should call useDaemonWebSocket hook on mount', () => {
      render(<DashboardPage />);

      expect(useDaemonWebSocket).toHaveBeenCalled();
    });

    it('should respond to WebSocket state changes', () => {
      const { rerender } = render(<DashboardPage />);

      // Initial state: disconnected
      expect(screen.getByText(/disconnected/i)).toBeInTheDocument();

      // Change to connecting
      vi.mocked(useDaemonWebSocket).mockReturnValue({
        readyState: ReadyState.CONNECTING,
        sendMessage: vi.fn(),
        lastMessage: null,
        isConnected: false,
        isConnecting: true,
        isDisconnected: false,
      });
      rerender(<DashboardPage />);
      expect(screen.getByText(/connecting to daemon/i)).toBeInTheDocument();

      // Change to connected
      vi.mocked(useDaemonWebSocket).mockReturnValue({
        readyState: ReadyState.OPEN,
        sendMessage: vi.fn(),
        lastMessage: null,
        isConnected: true,
        isConnecting: false,
        isDisconnected: false,
      });
      rerender(<DashboardPage />);
      expect(screen.getByText(/connected to daemon/i)).toBeInTheDocument();
    });
  });

  describe('Store Integration', () => {
    it('should read connection status from dashboard store', () => {
      useDashboardStore.getState().setConnectionStatus('connected');

      const { container } = render(<DashboardPage />);

      const banner = container.querySelector('.connection-banner');
      expect(banner).toHaveClass('connection-connected');
    });

    it('should reflect store updates in the UI', () => {
      const { rerender } = render(<DashboardPage />);

      // Initial: disconnected
      expect(screen.getByText(/disconnected/i)).toBeInTheDocument();

      // Update store
      useDashboardStore.getState().setConnectionStatus('connecting');
      vi.mocked(useDaemonWebSocket).mockReturnValue({
        readyState: ReadyState.CONNECTING,
        sendMessage: vi.fn(),
        lastMessage: null,
        isConnected: false,
        isConnecting: true,
        isDisconnected: false,
      });
      rerender(<DashboardPage />);

      expect(screen.getByText(/connecting to daemon/i)).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have proper heading hierarchy', () => {
      render(<DashboardPage />);

      const headings = screen.getAllByRole('heading', { level: 2 });
      expect(headings).toHaveLength(2); // Daemon State, Latency Metrics
      expect(headings[0]).toHaveTextContent('Daemon State');
      expect(headings[1]).toHaveTextContent('Latency Metrics');
    });

    it('should have semantic HTML structure', () => {
      const { container } = render(<DashboardPage />);

      expect(container.querySelector('.dashboard-page')).toBeInTheDocument();
      expect(container.querySelector('.dashboard-content')).toBeInTheDocument();
      expect(container.querySelector('.dashboard-grid')).toBeInTheDocument();
    });

    it('should have accessible connection status indicators', () => {
      render(<DashboardPage />);

      const statusIndicator = screen.getByText(/disconnected/i).previousElementSibling;
      expect(statusIndicator).toHaveClass('status-indicator');
    });
  });

  describe('Edge Cases', () => {
    it('should handle rapid connection state changes', () => {
      const { rerender } = render(<DashboardPage />);

      // Rapidly change states
      const states: ReadyState[] = [
        ReadyState.CONNECTING,
        ReadyState.OPEN,
        ReadyState.CLOSING,
        ReadyState.CLOSED,
      ];

      states.forEach((state) => {
        vi.mocked(useDaemonWebSocket).mockReturnValue({
          readyState: state,
          sendMessage: vi.fn(),
          lastMessage: null,
          isConnected: state === ReadyState.OPEN,
          isConnecting: state === ReadyState.CONNECTING,
          isDisconnected: state === ReadyState.CLOSED,
        });
        rerender(<DashboardPage />);
      });

      // Should render without crashing
      expect(screen.getByTestId('state-indicator-panel')).toBeInTheDocument();
    });

    it('should render correctly when all child components are present', () => {
      render(<DashboardPage />);

      // All three child components should be rendered
      expect(screen.getByTestId('state-indicator-panel')).toBeInTheDocument();
      expect(screen.getByTestId('metrics-chart')).toBeInTheDocument();
      expect(screen.getByTestId('event-timeline')).toBeInTheDocument();
    });
  });
});
