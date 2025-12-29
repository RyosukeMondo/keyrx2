import React from 'react';
import { ActiveProfileCard } from '../components/ActiveProfileCard';
import { DeviceListCard } from '../components/DeviceListCard';
import { QuickStatsCard } from '../components/QuickStatsCard';

/**
 * HomePage / Dashboard
 *
 * Main dashboard view showing:
 * - Active profile card with quick access to edit
 * - Connected devices list with status
 * - Quick statistics (latency, events, uptime)
 *
 * Layout: From design.md Layout 1
 * Responsive: Cards stack vertically on mobile (<768px)
 */
export const HomePage: React.FC = () => {
  return (
    <div className="flex flex-col gap-lg p-lg">
      <h1 className="text-2xl font-semibold text-slate-100">Dashboard</h1>

      {/* Active Profile Card */}
      <ActiveProfileCard />

      {/* Connected Devices Card */}
      <DeviceListCard />

      {/* Quick Stats Card */}
      <QuickStatsCard />
    </div>
  );
};
