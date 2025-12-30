import { describe, it, expect } from 'vitest';
import {
  formatTimestampMs,
  formatTimestampRelative,
  formatDuration,
} from './timeFormatting';

describe('formatTimestampMs', () => {
  it('should format microseconds as milliseconds (< 1000ms)', () => {
    expect(formatTimestampMs(500)).toBe('0.50ms');
    expect(formatTimestampMs(1500)).toBe('1.50ms');
    expect(formatTimestampMs(999000)).toBe('999.00ms');
  });

  it('should format microseconds as seconds (>= 1000ms)', () => {
    expect(formatTimestampMs(1000000)).toBe('1.000s');
    expect(formatTimestampMs(1500000)).toBe('1.500s');
    expect(formatTimestampMs(4567000)).toBe('4.567s');
    expect(formatTimestampMs(10000000)).toBe('10.000s');
  });

  it('should handle zero value', () => {
    expect(formatTimestampMs(0)).toBe('0.00ms');
  });

  it('should handle very small values', () => {
    expect(formatTimestampMs(1)).toBe('0.00ms');
    expect(formatTimestampMs(10)).toBe('0.01ms');
  });

  it('should handle very large values', () => {
    expect(formatTimestampMs(1000000000)).toBe('1000.000s');
    expect(formatTimestampMs(999999999999)).toBe('1000000.000s');
  });

  it('should handle negative values (edge case)', () => {
    expect(formatTimestampMs(-1000)).toBe('-1.00ms');
    expect(formatTimestampMs(-1000000)).toBe('-1000.00ms');
  });
});

describe('formatTimestampRelative', () => {
  const now = Date.now() / 1000; // Current time in seconds

  it('should return "Today" for timestamps from today', () => {
    const today = now;
    expect(formatTimestampRelative(today)).toBe('Today');
    expect(formatTimestampRelative(now - 3600)).toBe('Today'); // 1 hour ago
  });

  it('should return "Yesterday" for timestamps from yesterday', () => {
    const yesterday = now - 86400; // 24 hours ago
    expect(formatTimestampRelative(yesterday)).toBe('Yesterday');
  });

  it('should return days ago for timestamps < 7 days', () => {
    const twoDaysAgo = now - 2 * 86400;
    const threeDaysAgo = now - 3 * 86400;
    const sixDaysAgo = now - 6 * 86400;

    expect(formatTimestampRelative(twoDaysAgo)).toBe('2d ago');
    expect(formatTimestampRelative(threeDaysAgo)).toBe('3d ago');
    expect(formatTimestampRelative(sixDaysAgo)).toBe('6d ago');
  });

  it('should return weeks ago for timestamps < 30 days', () => {
    const oneWeekAgo = now - 7 * 86400;
    const twoWeeksAgo = now - 14 * 86400;
    const threeWeeksAgo = now - 21 * 86400;

    expect(formatTimestampRelative(oneWeekAgo)).toBe('1w ago');
    expect(formatTimestampRelative(twoWeeksAgo)).toBe('2w ago');
    expect(formatTimestampRelative(threeWeeksAgo)).toBe('3w ago');
  });

  it('should return locale date string for timestamps >= 30 days', () => {
    const thirtyDaysAgo = now - 30 * 86400;
    const oneYearAgo = now - 365 * 86400;

    const result30Days = formatTimestampRelative(thirtyDaysAgo);
    const result1Year = formatTimestampRelative(oneYearAgo);

    // Should be a date string (format depends on locale)
    expect(result30Days).toMatch(/\d+/);
    expect(result1Year).toMatch(/\d+/);
  });

  it('should handle zero timestamp', () => {
    const result = formatTimestampRelative(0);
    // Should return a date string for epoch time
    expect(result).toBeTruthy();
    expect(typeof result).toBe('string');
  });

  it('should handle negative timestamps (edge case)', () => {
    const result = formatTimestampRelative(-86400);
    expect(result).toBeTruthy();
    expect(typeof result).toBe('string');
  });

  it('should handle very large timestamps', () => {
    const farFuture = now + 365 * 86400; // 1 year in the future
    const result = formatTimestampRelative(farFuture);
    // Future timestamps will show as "Today" (0 days diff) or negative days
    expect(typeof result).toBe('string');
  });
});

describe('formatDuration', () => {
  it('should format microseconds as milliseconds (< 1000ms)', () => {
    expect(formatDuration(500)).toBe('1ms');
    expect(formatDuration(50000)).toBe('50ms');
    expect(formatDuration(123000)).toBe('123ms');
    expect(formatDuration(999000)).toBe('999ms');
  });

  it('should format microseconds as seconds (>= 1000ms)', () => {
    expect(formatDuration(1000000)).toBe('1.00s');
    expect(formatDuration(1500000)).toBe('1.50s');
    expect(formatDuration(2345000)).toBe('2.35s');
    expect(formatDuration(10000000)).toBe('10.00s');
  });

  it('should handle zero value', () => {
    expect(formatDuration(0)).toBe('0ms');
  });

  it('should handle very small values', () => {
    expect(formatDuration(1)).toBe('0ms');
    expect(formatDuration(10)).toBe('0ms');
    expect(formatDuration(999)).toBe('1ms');
  });

  it('should handle very large values', () => {
    expect(formatDuration(1000000000)).toBe('1000.00s');
    expect(formatDuration(999999999999)).toBe('1000000.00s');
  });

  it('should handle negative values (edge case)', () => {
    expect(formatDuration(-1000)).toBe('-1ms');
    expect(formatDuration(-1000000)).toBe('-1000ms');
  });

  it('should round milliseconds correctly', () => {
    expect(formatDuration(1234)).toBe('1ms'); // 1.234ms rounds to 1ms
    expect(formatDuration(4567)).toBe('5ms'); // 4.567ms rounds to 5ms
    expect(formatDuration(999499)).toBe('999ms'); // Just under 1000ms
  });

  it('should match existing EventTimeline format', () => {
    // EventTimeline uses toFixed(0) for ms and toFixed(2) for s
    expect(formatDuration(50000)).toBe('50ms');
    expect(formatDuration(1500000)).toBe('1.50s');
  });
});
