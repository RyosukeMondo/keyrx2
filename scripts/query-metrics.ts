#!/usr/bin/env node

/**
 * Query Metrics CLI
 *
 * Command-line tool for querying test metrics history.
 *
 * Usage:
 *   npx tsx scripts/query-metrics.ts <command> [options]
 *
 * Commands:
 *   report [max-entries]  Generate summary report (default: last 10 runs)
 *   latest                Show latest metrics entry
 *   clear                 Clear all metrics
 */

import { TestMetrics } from './metrics/test-metrics.js';

async function main() {
  const command = process.argv[2];
  const metricsFile = process.env.METRICS_FILE || 'metrics.jsonl';

  if (!command || command === '--help' || command === '-h') {
    console.log(`
Query Metrics CLI

Usage: npx tsx scripts/query-metrics.ts <command> [options]

Commands:
  report [max-entries]  Generate summary report (default: last 10 runs)
  latest                Show latest metrics entry
  clear                 Clear all metrics

Environment:
  METRICS_FILE          Path to metrics file (default: metrics.jsonl)
    `);
    process.exit(0);
  }

  const metrics = new TestMetrics(metricsFile);

  switch (command) {
    case 'report': {
      const maxEntries = parseInt(process.argv[3] || '10', 10);
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
      console.error(`Unknown command: ${command}`);
      console.error('Run with --help for usage information');
      process.exit(1);
  }
}

main().catch((error) => {
  console.error('Error:', error.message);
  process.exit(1);
});
