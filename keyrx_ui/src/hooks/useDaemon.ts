/**
 * Hook for daemon control operations.
 *
 * Provides functionality for controlling the daemon process,
 * including restart operations.
 */

import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useUnifiedApi } from './useUnifiedApi';
import { RpcClient } from '../api/rpc';

/**
 * Hook for restarting the daemon.
 *
 * Returns a mutation that can be used to restart the daemon process.
 * The daemon will restart with the currently active profile, applying
 * any configuration changes.
 *
 * @example
 * ```tsx
 * function RestartButton() {
 *   const restartMutation = useRestartDaemon();
 *
 *   return (
 *     <button
 *       onClick={() => restartMutation.mutate()}
 *       disabled={restartMutation.isPending}
 *     >
 *       {restartMutation.isPending ? 'Restarting...' : 'Restart Daemon'}
 *     </button>
 *   );
 * }
 * ```
 */
export function useRestartDaemon() {
  const api = useUnifiedApi();
  const queryClient = useQueryClient();
  const client = new RpcClient(api);

  return useMutation({
    mutationFn: async () => {
      const result = await client.restartDaemon();
      return result;
    },
    onSuccess: () => {
      // The daemon will restart, so the WebSocket will disconnect.
      // The reconnection logic will handle reconnecting automatically.
      // Invalidate all queries so they'll be refetched after reconnect.
      setTimeout(() => {
        queryClient.invalidateQueries();
      }, 2000); // Give the daemon time to restart
    },
  });
}

/**
 * Hook for activating a profile and restarting the daemon.
 *
 * This is a convenience hook that combines profile activation with
 * a daemon restart to immediately apply configuration changes.
 *
 * @example
 * ```tsx
 * function ActivateButton({ profileName }) {
 *   const activateAndRestart = useActivateProfileAndRestart();
 *
 *   return (
 *     <button
 *       onClick={() => activateAndRestart.mutate(profileName)}
 *       disabled={activateAndRestart.isPending}
 *     >
 *       Activate & Apply
 *     </button>
 *   );
 * }
 * ```
 */
export function useActivateProfileAndRestart() {
  const api = useUnifiedApi();
  const queryClient = useQueryClient();
  const client = new RpcClient(api);

  return useMutation({
    mutationFn: async (profileName: string) => {
      await client.activateProfileAndRestart(profileName);
    },
    onSuccess: () => {
      // Invalidate queries after restart
      setTimeout(() => {
        queryClient.invalidateQueries();
      }, 2000);
    },
  });
}
