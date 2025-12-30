/**
 * Time formatting utilities for consistent timestamp display across the application.
 *
 * This module provides pure functions for formatting timestamps and durations
 * in various human-readable formats. All functions are stateless and have no side effects.
 *
 * @module timeFormatting
 */

/**
 * Formats a timestamp in microseconds to a human-readable string (ms/s).
 *
 * @param timestampUs - Timestamp in microseconds
 * @returns Formatted string like "1.23ms" or "4.567s"
 *
 * @example
 * formatTimestampMs(500)      // "0.50ms"
 * formatTimestampMs(1500)     // "1.50ms"
 * formatTimestampMs(1500000)  // "1.500s"
 */
export function formatTimestampMs(timestampUs: number): string {
  const ms = timestampUs / 1000;
  if (ms < 1000) {
    return `${ms.toFixed(2)}ms`;
  }
  const seconds = ms / 1000;
  return `${seconds.toFixed(3)}s`;
}

/**
 * Formats a timestamp (in seconds since epoch) to a relative time string.
 *
 * @param timestamp - Unix timestamp in seconds
 * @returns Relative time string like "Today", "Yesterday", "3d ago", "2w ago", or a date
 *
 * @example
 * formatTimestampRelative(Date.now() / 1000)           // "Today"
 * formatTimestampRelative(Date.now() / 1000 - 86400)   // "Yesterday"
 * formatTimestampRelative(Date.now() / 1000 - 259200)  // "3d ago"
 */
export function formatTimestampRelative(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays === 0) {
    return 'Today';
  } else if (diffDays === 1) {
    return 'Yesterday';
  } else if (diffDays < 7) {
    return `${diffDays}d ago`;
  } else if (diffDays < 30) {
    return `${Math.floor(diffDays / 7)}w ago`;
  } else {
    return date.toLocaleDateString();
  }
}

/**
 * Formats a duration in microseconds to a human-readable string.
 *
 * @param durationUs - Duration in microseconds
 * @returns Formatted string like "123ms" or "1.23s"
 *
 * @example
 * formatDuration(500)      // "0ms"
 * formatDuration(50000)    // "50ms"
 * formatDuration(1500000)  // "1.50s"
 */
export function formatDuration(durationUs: number): string {
  const ms = durationUs / 1000;
  if (ms < 1000) {
    return `${ms.toFixed(0)}ms`;
  }
  return `${(ms / 1000).toFixed(2)}s`;
}
