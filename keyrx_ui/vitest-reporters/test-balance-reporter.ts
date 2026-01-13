/**
 * Vitest Custom Reporter: Test Balance Reporter
 *
 * Monitors test distribution across test categories (unit, integration, E2E)
 * and warns if the distribution deviates significantly from the test pyramid strategy.
 *
 * Target distribution (from tests/README.md):
 * - Unit: 70%
 * - Integration: 20%
 * - E2E: 10%
 *
 * Warning threshold: 80/15/5 deviation
 * - Unit: < 65% or > 85%
 * - Integration: < 15% or > 30%
 * - E2E: < 5% or > 15%
 */

import type { Reporter, File, TaskResult } from 'vitest';

interface TestCounts {
  unit: number;
  integration: number;
  e2e: number;
  total: number;
}

export default class TestBalanceReporter implements Reporter {
  private counts: TestCounts = {
    unit: 0,
    integration: 0,
    e2e: 0,
    total: 0,
  };

  onInit() {
    this.counts = { unit: 0, integration: 0, e2e: 0, total: 0 };
  }

  onCollected(files?: File[]) {
    if (!files) return;

    // Count tests by category based on filename
    for (const file of files) {
      const testCount = this.countTests(file);

      if (file.filepath.includes('.e2e.')) {
        this.counts.e2e += testCount;
      } else if (file.filepath.includes('.integration.test.')) {
        this.counts.integration += testCount;
      } else if (file.filepath.includes('.test.')) {
        this.counts.unit += testCount;
      }

      this.counts.total += testCount;
    }
  }

  onFinished(files?: File[], errors?: unknown[]) {
    // Recalculate to ensure accuracy
    this.counts = { unit: 0, integration: 0, e2e: 0, total: 0 };

    if (files) {
      for (const file of files) {
        const testCount = this.countTests(file);

        if (file.filepath.includes('.e2e.')) {
          this.counts.e2e += testCount;
        } else if (file.filepath.includes('.integration.test.')) {
          this.counts.integration += testCount;
        } else if (file.filepath.includes('.test.')) {
          this.counts.unit += testCount;
        }

        this.counts.total += testCount;
      }
    }

    this.printReport();
  }

  private countTests(file: File): number {
    let count = 0;

    const traverse = (task: File | TaskResult): void => {
      if ('tasks' in task && task.tasks) {
        task.tasks.forEach(traverse);
      }
      if (task.type === 'test') {
        count++;
      }
    };

    traverse(file);
    return count;
  }

  private printReport() {
    if (this.counts.total === 0) {
      return;
    }

    const unitPct = (this.counts.unit / this.counts.total) * 100;
    const integrationPct = (this.counts.integration / this.counts.total) * 100;
    const e2ePct = (this.counts.e2e / this.counts.total) * 100;

    console.log('\n');
    console.log('═══════════════════════════════════════════════════════');
    console.log('  Test Pyramid Balance Report');
    console.log('═══════════════════════════════════════════════════════');
    console.log('');
    console.log('  Category       Count    Percentage    Target    Status');
    console.log('  ───────────────────────────────────────────────────────');
    console.log(
      `  Unit           ${this.pad(this.counts.unit)}    ${this.formatPct(unitPct)}    70%       ${this.getStatus(unitPct, 65, 85)}`
    );
    console.log(
      `  Integration    ${this.pad(this.counts.integration)}    ${this.formatPct(integrationPct)}    20%       ${this.getStatus(integrationPct, 15, 30)}`
    );
    console.log(
      `  E2E            ${this.pad(this.counts.e2e)}    ${this.formatPct(e2ePct)}    10%       ${this.getStatus(e2ePct, 5, 15)}`
    );
    console.log('  ───────────────────────────────────────────────────────');
    console.log(`  Total          ${this.pad(this.counts.total)}`);
    console.log('');

    // Check for warnings
    const warnings: string[] = [];

    if (unitPct < 65) {
      warnings.push(`⚠️  Unit test percentage (${unitPct.toFixed(1)}%) is below target (65-85%)`);
    } else if (unitPct > 85) {
      warnings.push(`⚠️  Unit test percentage (${unitPct.toFixed(1)}%) is above target (65-85%)`);
    }

    if (integrationPct < 15) {
      warnings.push(`⚠️  Integration test percentage (${integrationPct.toFixed(1)}%) is below target (15-30%)`);
    } else if (integrationPct > 30) {
      warnings.push(`⚠️  Integration test percentage (${integrationPct.toFixed(1)}%) is above target (15-30%)`);
    }

    if (e2ePct < 5) {
      warnings.push(`⚠️  E2E test percentage (${e2ePct.toFixed(1)}%) is below target (5-15%)`);
    } else if (e2ePct > 15) {
      warnings.push(`⚠️  E2E test percentage (${e2ePct.toFixed(1)}%) is above target (5-15%)`);
    }

    if (warnings.length > 0) {
      console.log('  Test Balance Warnings:');
      console.log('');
      warnings.forEach((warning) => console.log(`  ${warning}`));
      console.log('');
      console.log('  Recommendation: Review tests/README.md for test pyramid guidelines');
      console.log('');
    } else {
      console.log('  ✓ Test distribution is within acceptable ranges');
      console.log('');
    }

    console.log('═══════════════════════════════════════════════════════');
    console.log('');
  }

  private pad(num: number, width: number = 4): string {
    const str = num.toString();
    return str.padStart(width, ' ');
  }

  private formatPct(pct: number): string {
    return pct.toFixed(1).padStart(5, ' ') + '%';
  }

  private getStatus(pct: number, min: number, max: number): string {
    if (pct < min) return '⚠️  Low';
    if (pct > max) return '⚠️  High';
    return '✓ OK';
  }
}
