import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import { QuickStatsCard } from './QuickStatsCard';

describe('QuickStatsCard', () => {
  const mockStats = {
    latencyAvg: 2.3,
    eventsToday: 1247,
    uptimeSeconds: 19380, // 5h 23m
  };

  it('renders loading state', () => {
    renderWithProviders(<QuickStatsCard loading={true} />);
    const loadingElements = screen.getAllByRole('generic');
    const hasAnimatePulse = loadingElements.some((el) =>
      el.classList.contains('animate-pulse')
    );
    expect(hasAnimatePulse).toBe(true);
  });

  it('renders empty state when no stats', () => {
    renderWithProviders(<QuickStatsCard />);
    expect(screen.getByText('Quick Stats')).toBeInTheDocument();
    expect(
      screen.getByText(/Statistics unavailable/)
    ).toBeInTheDocument();
  });

  it('renders stats heading', () => {
    renderWithProviders(<QuickStatsCard stats={mockStats} />);
    expect(screen.getByText('Quick Stats')).toBeInTheDocument();
  });

  it('renders latency with correct precision', () => {
    renderWithProviders(<QuickStatsCard stats={mockStats} />);
    expect(screen.getByText(/2.3ms avg/)).toBeInTheDocument();
  });

  it('formats event count with thousands separator', () => {
    renderWithProviders(<QuickStatsCard stats={mockStats} />);
    expect(screen.getByText(/1,247 today/)).toBeInTheDocument();
  });

  it('formats large event numbers correctly', () => {
    const largeStats = {
      ...mockStats,
      eventsToday: 123456,
    };
    renderWithProviders(<QuickStatsCard stats={largeStats} />);
    expect(screen.getByText(/123,456 today/)).toBeInTheDocument();
  });

  it('formats uptime correctly in hours and minutes', () => {
    renderWithProviders(<QuickStatsCard stats={mockStats} />);
    expect(screen.getByText(/5h 23m/)).toBeInTheDocument();
  });

  it('handles uptime less than 1 hour', () => {
    const shortUptime = {
      ...mockStats,
      uptimeSeconds: 1800, // 30 minutes
    };
    renderWithProviders(<QuickStatsCard stats={shortUptime} />);
    expect(screen.getByText(/0h 30m/)).toBeInTheDocument();
  });

  it('handles uptime with exact hours (no minutes)', () => {
    const exactHours = {
      ...mockStats,
      uptimeSeconds: 7200, // 2 hours exactly
    };
    renderWithProviders(<QuickStatsCard stats={exactHours} />);
    expect(screen.getByText(/2h 0m/)).toBeInTheDocument();
  });

  it('renders latency label with correct styling', () => {
    renderWithProviders(<QuickStatsCard stats={mockStats} />);
    const latencyLabel = screen.getByText('Latency:');
    expect(latencyLabel).toHaveClass('font-medium', 'text-slate-100');
  });

  it('renders events label with correct styling', () => {
    renderWithProviders(<QuickStatsCard stats={mockStats} />);
    const eventsLabel = screen.getByText('Events:');
    expect(eventsLabel).toHaveClass('font-medium', 'text-slate-100');
  });

  it('renders uptime label with correct styling', () => {
    renderWithProviders(<QuickStatsCard stats={mockStats} />);
    const uptimeLabel = screen.getByText('Uptime:');
    expect(uptimeLabel).toHaveClass('font-medium', 'text-slate-100');
  });

  it('uses bullet separators between stats', () => {
    renderWithProviders(<QuickStatsCard stats={mockStats} />);
    const bullets = screen.getAllByText('•');
    expect(bullets).toHaveLength(2);
  });

  it('applies custom className', () => {
    const { container } = renderWithProviders(
      <QuickStatsCard stats={mockStats} className="custom-class" />
    );
    expect(container.querySelector('.custom-class')).toBeInTheDocument();
  });

  it('handles zero latency', () => {
    const zeroLatency = {
      ...mockStats,
      latencyAvg: 0,
    };
    renderWithProviders(<QuickStatsCard stats={zeroLatency} />);
    expect(screen.getByText(/0.0ms avg/)).toBeInTheDocument();
  });

  it('handles high latency values', () => {
    const highLatency = {
      ...mockStats,
      latencyAvg: 15.7,
    };
    renderWithProviders(<QuickStatsCard stats={highLatency} />);
    expect(screen.getByText(/15.7ms avg/)).toBeInTheDocument();
  });
});
