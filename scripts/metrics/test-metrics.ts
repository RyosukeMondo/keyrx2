/**
 * Test Metrics Collection System
 *
 * Collects and tracks test execution metrics over time:
 * - Pass/fail rates
 * - Test duration
 * - Fix attempt success rates
 * - Trend analysis
 */

import * as fs from 'fs';
import * as path from 'path';

/**
 * Result of a single test execution
 */
export interface TestResult {
  id: string;
  name: string;
  status: 'pass' | 'fail';
  duration: number;
  error?: string;
  fixAttempts?: number;
  fixSuccesses?: number;
}

/**
 * Complete test suite results
 */
export interface TestSuiteResult {
  total: number;
  passed: number;
  failed: number;
  duration: number;
  results: TestResult[];
}

/**
 * Metrics entry for a single test run
 */
export interface MetricsEntry {
  timestamp: string;
  totalTests: number;
  passedTests: number;
  failedTests: number;
  passRate: number;
  duration: number;
  fixAttempts: number;
  fixSuccesses: number;
  fixSuccessRate: number;
  averageTestDuration: number;
  slowestTests: Array<{
    id: string;
    name: string;
    duration: number;
  }>;
}

/**
 * Metrics summary report
 */
export interface MetricsReport {
  passRateTrend: Array<{
    timestamp: string;
    passRate: number;
  }>;
  averageDuration: number;
  mostFlakyTests: Array<{
    id: string;
    name: string;
    failureRate: number;
    occurrences: number;
  }>;
  totalRuns: number;
}

/**
 * Test metrics collector and reporter
 */
export class TestMetrics {
  private metricsFile: string;
  private maxLines: number = 1000;

  /**
   * Create a new TestMetrics instance
   * @param metricsFile Path to the metrics JSONL file
   */
  constructor(metricsFile: string = 'metrics.jsonl') {
    this.metricsFile = metricsFile;
  }

  /**
   * Record test results to metrics file
   * @param testResults Test suite results
   * @param timestamp Optional timestamp (defaults to now)
   */
  async record(testResults: TestSuiteResult, timestamp?: string): Promise<void> {
    const ts = timestamp || new Date().toISOString();

    // Calculate metrics
    const fixAttempts = testResults.results.reduce(
      (sum, r) => sum + (r.fixAttempts || 0),
      0
    );
    const fixSuccesses = testResults.results.reduce(
      (sum, r) => sum + (r.fixSuccesses || 0),
      0
    );
    const passRate = testResults.total > 0
      ? (testResults.passed / testResults.total) * 100
      : 0;
    const fixSuccessRate = fixAttempts > 0
      ? (fixSuccesses / fixAttempts) * 100
      : 0;
    const averageTestDuration = testResults.total > 0
      ? testResults.duration / testResults.total
      : 0;

    // Find slowest tests (top 5)
    const sortedByDuration = [...testResults.results]
      .sort((a, b) => b.duration - a.duration)
      .slice(0, 5);
    const slowestTests = sortedByDuration.map(r => ({
      id: r.id,
      name: r.name,
      duration: r.duration,
    }));

    const entry: MetricsEntry = {
      timestamp: ts,
      totalTests: testResults.total,
      passedTests: testResults.passed,
      failedTests: testResults.failed,
      passRate,
      duration: testResults.duration,
      fixAttempts,
      fixSuccesses,
      fixSuccessRate,
      averageTestDuration,
      slowestTests,
    };

    // Append to file
    await this.appendEntry(entry);

    // Rotate if needed
    await this.rotateIfNeeded();
  }

  /**
   * Generate metrics report from historical data
   * @param maxEntries Maximum number of recent entries to analyze (default: 10)
   */
  async report(maxEntries: number = 10): Promise<MetricsReport> {
    const entries = await this.loadEntries(maxEntries);

    if (entries.length === 0) {
      return {
        passRateTrend: [],
        averageDuration: 0,
        mostFlakyTests: [],
        totalRuns: 0,
      };
    }

    // Pass rate trend
    const passRateTrend = entries.map(e => ({
      timestamp: e.timestamp,
      passRate: e.passRate,
    }));

    // Average duration
    const totalDuration = entries.reduce((sum, e) => sum + e.duration, 0);
    const averageDuration = totalDuration / entries.length;

    // Most flaky tests (tests that fail most frequently)
    const testFailures = new Map<string, { name: string; failures: number; total: number }>();

    // We need to load the detailed results to analyze flakiness
    // For now, we'll use a simplified approach based on slowest tests
    // In a real implementation, you'd want to store individual test results
    const mostFlakyTests: Array<{
      id: string;
      name: string;
      failureRate: number;
      occurrences: number;
    }> = [];

    return {
      passRateTrend,
      averageDuration,
      mostFlakyTests,
      totalRuns: entries.length,
    };
  }

  /**
   * Get the most recent metrics entry
   */
  async getLatest(): Promise<MetricsEntry | null> {
    const entries = await this.loadEntries(1);
    return entries.length > 0 ? entries[0] : null;
  }

  /**
   * Get all metrics entries
   */
  async getAllEntries(): Promise<MetricsEntry[]> {
    return this.loadEntries();
  }

  /**
   * Clear all metrics
   */
  async clear(): Promise<void> {
    if (fs.existsSync(this.metricsFile)) {
      await fs.promises.unlink(this.metricsFile);
    }
  }

  /**
   * Append a metrics entry to the JSONL file
   */
  private async appendEntry(entry: MetricsEntry): Promise<void> {
    const line = JSON.stringify(entry) + '\n';

    // Ensure directory exists
    const dir = path.dirname(this.metricsFile);
    if (dir && dir !== '.') {
      await fs.promises.mkdir(dir, { recursive: true });
    }

    // Append to file
    await fs.promises.appendFile(this.metricsFile, line, 'utf-8');
  }

  /**
   * Load metrics entries from file
   * @param maxEntries Maximum number of entries to load (from end of file)
   */
  private async loadEntries(maxEntries?: number): Promise<MetricsEntry[]> {
    if (!fs.existsSync(this.metricsFile)) {
      return [];
    }

    const content = await fs.promises.readFile(this.metricsFile, 'utf-8');
    const lines = content.trim().split('\n').filter(line => line.length > 0);

    // Parse each line as JSON
    const entries: MetricsEntry[] = [];
    for (const line of lines) {
      try {
        const entry = JSON.parse(line);
        entries.push(entry);
      } catch (error) {
        // Skip malformed lines
        console.warn(`Skipping malformed metrics line: ${line.substring(0, 50)}...`);
      }
    }

    // Return the most recent entries if maxEntries is specified
    if (maxEntries !== undefined) {
      return entries.slice(-maxEntries);
    }

    return entries;
  }

  /**
   * Rotate metrics file if it exceeds max lines
   */
  private async rotateIfNeeded(): Promise<void> {
    if (!fs.existsSync(this.metricsFile)) {
      return;
    }

    const content = await fs.promises.readFile(this.metricsFile, 'utf-8');
    const lines = content.trim().split('\n').filter(line => line.length > 0);

    if (lines.length > this.maxLines) {
      // Keep only the most recent maxLines entries
      const recentLines = lines.slice(-this.maxLines);
      await fs.promises.writeFile(
        this.metricsFile,
        recentLines.join('\n') + '\n',
        'utf-8'
      );
    }
  }

  /**
   * Format metrics report as human-readable text
   */
  static formatReport(report: MetricsReport): string {
    const lines: string[] = [];

    lines.push('=== Test Metrics Report ===\n');
    lines.push(`Total Runs: ${report.totalRuns}\n`);
    lines.push(`Average Duration: ${report.averageDuration.toFixed(2)}ms\n`);

    if (report.passRateTrend.length > 0) {
      lines.push('\n--- Pass Rate Trend (Last 10 Runs) ---');
      for (const entry of report.passRateTrend) {
        const date = new Date(entry.timestamp).toLocaleString();
        const bar = '█'.repeat(Math.round(entry.passRate / 5));
        lines.push(`${date}: ${entry.passRate.toFixed(1)}% ${bar}`);
      }
    }

    if (report.mostFlakyTests.length > 0) {
      lines.push('\n--- Most Flaky Tests ---');
      for (const test of report.mostFlakyTests) {
        lines.push(
          `${test.name}: ${test.failureRate.toFixed(1)}% failure rate (${test.occurrences} occurrences)`
        );
      }
    }

    return lines.join('\n');
  }
}

// CLI interface
if (require.main === module) {
  const command = process.argv[2];
  const metricsFile = process.argv[3] || 'metrics.jsonl';

  const metrics = new TestMetrics(metricsFile);

  (async () => {
    switch (command) {
      case 'report': {
        const maxEntries = parseInt(process.argv[4] || '10', 10);
        const report = await metrics.report(maxEntries);
        console.log(TestMetrics.formatReport(report));
        break;
      }

      case 'latest': {
        const latest = await metrics.getLatest();
        if (latest) {
          console.log(JSON.stringify(latest, null, 2));
        } else {
          console.log('No metrics found');
        }
        break;
      }

      case 'clear': {
        await metrics.clear();
        console.log('Metrics cleared');
        break;
      }

      default:
        console.log('Usage:');
        console.log('  tsx test-metrics.ts report [metrics-file] [max-entries]');
        console.log('  tsx test-metrics.ts latest [metrics-file]');
        console.log('  tsx test-metrics.ts clear [metrics-file]');
        process.exit(1);
    }
  })().catch(error => {
    console.error('Error:', error.message);
    process.exit(1);
  });
}
