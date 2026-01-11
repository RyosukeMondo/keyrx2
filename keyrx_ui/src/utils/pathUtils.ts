/**
 * Path utility functions for displaying file paths
 */

/**
 * Truncates a long file path with ellipsis in the middle
 * Example: /very/long/path/to/file.txt -> /very/.../file.txt
 */
export function truncatePath(path: string, maxLength: number = 50): string {
  // Replace home directory with ~ first
  const homeReplaced = path.replace(/^\/home\/[^/]+/, '~');

  // If short enough after home replacement, return it
  if (homeReplaced.length <= maxLength) {
    return homeReplaced;
  }

  // Split into parts
  const parts = homeReplaced.split('/');
  if (parts.length <= 2) {
    // Can't truncate much, just cut from start
    return '...' + homeReplaced.slice(-(maxLength - 3));
  }

  // Keep first and last parts, add ellipsis in middle
  const first = parts.slice(0, 2).join('/');
  const last = parts[parts.length - 1];
  const truncated = `${first}/.../${last}`;

  if (truncated.length <= maxLength) {
    return truncated;
  }

  // If still too long, just truncate from start
  return '...' + homeReplaced.slice(-(maxLength - 3));
}

/**
 * Converts absolute path to user-friendly format
 * - Replaces /home/username with ~
 * - Shortens config paths
 */
export function formatPathForDisplay(path: string): string {
  return path
    .replace(/^\/home\/[^/]+/, '~')
    .replace(/\.config\/keyrx\/profiles\//, '');
}

/**
 * Checks if a file path exists (client-side approximation)
 * In practice, this should be determined by the backend
 */
export function isPathAccessible(path: string): boolean {
  // This is a placeholder - actual file existence should be checked by backend
  // For now, we assume all paths returned by the API are valid
  return path.length > 0;
}
