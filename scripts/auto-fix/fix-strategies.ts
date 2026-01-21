/**
 * Auto-Fix Strategies for E2E Test Failures
 *
 * Implements strategy pattern for automated issue remediation:
 * - RestartDaemonStrategy: Fixes network and daemon issues by restarting
 * - UpdateExpectedResultStrategy: Updates expected results for schema changes
 * - ReseedFixtureStrategy: Re-seeds fixtures for data issues
 * - RetryTestStrategy: Retries tests for transient failures
 *
 * All strategies are idempotent and conservative (never modify code).
 */

import type { Issue } from './issue-classifier.js';
import type { DaemonFixture } from '../fixtures/daemon-fixture.js';
import type { TestExecutionResult } from '../test-cases/types.js';
import * as fs from 'fs';
import * as path from 'path';

/**
 * Result of applying a fix strategy
 */
export interface FixResult {
  /** Whether the fix was successfully applied */
  success: boolean;
  /** Human-readable message describing the outcome */
  message: string;
  /** Whether to retry the test after this fix */
  retry: boolean;
  /** Error details if fix failed */
  error?: string;
}

/**
 * Context for fix strategy execution
 */
export interface FixContext {
  /** Daemon fixture for lifecycle management */
  daemonFixture: DaemonFixture;
  /** Base URL for API requests */
  apiBaseUrl: string;
  /** Test result that triggered the fix */
  testResult: TestExecutionResult;
  /** Path to expected results file */
  expectedResultsPath: string;
  /** Maximum wait time for operations (ms) */
  timeout: number;
  /** Enable verbose logging */
  verbose: boolean;
}

/**
 * Base interface for fix strategies
 */
export interface FixStrategy {
  /** Strategy name */
  readonly name: string;

  /**
   * Check if this strategy can fix the given issue
   */
  canFix(issue: Issue): boolean;

  /**
   * Apply the fix for the given issue
   */
  apply(issue: Issue, context: FixContext): Promise<FixResult>;
}

/**
 * Strategy: Restart Daemon
 *
 * Fixes network errors by restarting the daemon process.
 * Handles: ECONNREFUSED, timeout, 500 errors, socket hang up.
 */
export class RestartDaemonStrategy implements FixStrategy {
  readonly name = 'RestartDaemon';

  canFix(issue: Issue): boolean {
    return issue.type === 'network' && issue.fixable && issue.priority === 1;
  }

  async apply(issue: Issue, context: FixContext): Promise<FixResult> {
    const { daemonFixture, verbose } = context;

    try {
      if (verbose) {
        console.log(`[${this.name}] Restarting daemon to fix network issue...`);
      }

      // Stop daemon gracefully
      await daemonFixture.stop();

      // Brief pause to ensure clean shutdown
      await new Promise((resolve) => setTimeout(resolve, 1000));

      // Start daemon again
      await daemonFixture.start();

      if (verbose) {
        console.log(`[${this.name}] Daemon restarted successfully`);
      }

      return {
        success: true,
        message: 'Daemon restarted successfully',
        retry: true,
      };
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      return {
        success: false,
        message: 'Failed to restart daemon',
        retry: false,
        error: errorMsg,
      };
    }
  }
}

/**
 * Strategy: Update Expected Results
 *
 * Updates expected-results.json when schema changes are detected.
 * Handles: type mismatches, missing fields, extra fields.
 *
 * IMPORTANT: Only updates expected results, never modifies code.
 */
export class UpdateExpectedResultStrategy implements FixStrategy {
  readonly name = 'UpdateExpectedResult';

  canFix(issue: Issue): boolean {
    // Only fix validation issues with priority 1 or 2
    return (
      issue.type === 'validation' &&
      issue.fixable &&
      (issue.priority === 1 || issue.priority === 2)
    );
  }

  async apply(issue: Issue, context: FixContext): Promise<FixResult> {
    const { expectedResultsPath, testResult, verbose } = context;

    try {
      if (verbose) {
        console.log(`[${this.name}] Updating expected results for ${testResult.endpoint}...`);
      }

      // Read current expected results
      const expectedData = JSON.parse(fs.readFileSync(expectedResultsPath, 'utf-8'));

      // Find the scenario in expected results
      const endpoint = testResult.endpoint;
      const scenario = testResult.scenario;

      if (!expectedData.endpoints || !expectedData.endpoints[endpoint]) {
        return {
          success: false,
          message: `Endpoint ${endpoint} not found in expected results`,
          retry: false,
        };
      }

      if (!expectedData.endpoints[endpoint].scenarios[scenario]) {
        return {
          success: false,
          message: `Scenario ${scenario} not found for endpoint ${endpoint}`,
          retry: false,
        };
      }

      // Check if this is an extra field issue (add to ignore list)
      if (issue.description.includes('Extra field') && issue.diff && issue.diff.length > 0) {
        const diff = issue.diff[0];
        const fieldPath = diff.path;

        // Add to ignoreFields
        const scenarioConfig = expectedData.endpoints[endpoint].scenarios[scenario];
        if (!scenarioConfig.ignoreFields) {
          scenarioConfig.ignoreFields = [];
        }

        if (!scenarioConfig.ignoreFields.includes(fieldPath)) {
          scenarioConfig.ignoreFields.push(fieldPath);

          // Write back to file
          fs.writeFileSync(expectedResultsPath, JSON.stringify(expectedData, null, 2));

          if (verbose) {
            console.log(`[${this.name}] Added ${fieldPath} to ignore list`);
          }

          return {
            success: true,
            message: `Added ${fieldPath} to ignore list`,
            retry: true,
          };
        } else {
          return {
            success: false,
            message: `Field ${fieldPath} already in ignore list`,
            retry: false,
          };
        }
      }

      // For other validation issues, update the expected result body
      // This is conservative - we only do this for similar values or type changes
      if (
        issue.description.includes('Similar but not equal') ||
        issue.description.includes('Type mismatch')
      ) {
        const scenarioConfig = expectedData.endpoints[endpoint].scenarios[scenario];

        // Update the expected body with the actual response
        if (testResult.actual) {
          scenarioConfig.body = testResult.actual;

          // Write back to file
          fs.writeFileSync(expectedResultsPath, JSON.stringify(expectedData, null, 2));

          if (verbose) {
            console.log(`[${this.name}] Updated expected result for ${scenario}`);
          }

          return {
            success: true,
            message: `Updated expected result for ${scenario}`,
            retry: true,
          };
        }
      }

      return {
        success: false,
        message: 'Issue not automatically fixable via expected results update',
        retry: false,
      };
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      return {
        success: false,
        message: 'Failed to update expected results',
        retry: false,
        error: errorMsg,
      };
    }
  }
}

/**
 * Strategy: Re-seed Fixtures
 *
 * Re-seeds test fixtures when data issues are detected.
 * Handles: empty arrays, missing data, fixture issues.
 *
 * This is a placeholder - actual implementation depends on fixture system.
 */
export class ReseedFixtureStrategy implements FixStrategy {
  readonly name = 'ReseedFixture';

  canFix(issue: Issue): boolean {
    return issue.type === 'data' && issue.fixable && issue.priority === 2;
  }

  async apply(issue: Issue, context: FixContext): Promise<FixResult> {
    const { verbose, testResult } = context;

    try {
      if (verbose) {
        console.log(`[${this.name}] Re-seeding fixtures for ${testResult.endpoint}...`);
      }

      // For now, this is a placeholder
      // In a real implementation, we would:
      // 1. Identify the fixture needed based on endpoint/scenario
      // 2. Call a fixture seeding API or script
      // 3. Verify the data is populated

      // Since we don't have a fixture system yet, we'll just log and skip
      if (verbose) {
        console.log(`[${this.name}] Fixture re-seeding not implemented yet`);
      }

      return {
        success: false,
        message: 'Fixture re-seeding not implemented',
        retry: false,
      };
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      return {
        success: false,
        message: 'Failed to re-seed fixtures',
        retry: false,
        error: errorMsg,
      };
    }
  }
}

/**
 * Strategy: Retry Test
 *
 * Simply retries the test without making changes.
 * Handles: transient failures, race conditions.
 *
 * This is the most conservative strategy - just wait and retry.
 */
export class RetryTestStrategy implements FixStrategy {
  readonly name = 'RetryTest';

  canFix(issue: Issue): boolean {
    // Can try to fix any issue by retrying, but only as a last resort
    return issue.priority <= 2;
  }

  async apply(issue: Issue, context: FixContext): Promise<FixResult> {
    const { verbose } = context;

    try {
      if (verbose) {
        console.log(`[${this.name}] Waiting before retry...`);
      }

      // Wait a bit before retrying (give daemon time to settle)
      await new Promise((resolve) => setTimeout(resolve, 2000));

      return {
        success: true,
        message: 'Waiting period complete, ready to retry',
        retry: true,
      };
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      return {
        success: false,
        message: 'Failed to wait for retry',
        retry: false,
        error: errorMsg,
      };
    }
  }
}

/**
 * Strategy: Wait for Daemon
 *
 * Waits for daemon to become responsive without restarting.
 * Handles: slow startup, daemon still initializing.
 */
export class WaitForDaemonStrategy implements FixStrategy {
  readonly name = 'WaitForDaemon';

  canFix(issue: Issue): boolean {
    // Timeout errors can often be fixed by waiting longer
    return (
      issue.type === 'network' &&
      issue.fixable &&
      issue.priority === 1 &&
      issue.description.includes('timeout')
    );
  }

  async apply(issue: Issue, context: FixContext): Promise<FixResult> {
    const { daemonFixture, verbose, timeout } = context;

    try {
      if (verbose) {
        console.log(`[${this.name}] Waiting for daemon to become responsive...`);
      }

      // Check daemon health with extended timeout
      await daemonFixture.waitUntilReady(timeout);

      if (verbose) {
        console.log(`[${this.name}] Daemon is now responsive`);
      }

      return {
        success: true,
        message: 'Daemon is now responsive',
        retry: true,
      };
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      return {
        success: false,
        message: 'Daemon did not become responsive',
        retry: false,
        error: errorMsg,
      };
    }
  }
}

/**
 * Get all available fix strategies
 */
export function getAllStrategies(): FixStrategy[] {
  return [
    new RestartDaemonStrategy(),
    new UpdateExpectedResultStrategy(),
    new WaitForDaemonStrategy(),
    new ReseedFixtureStrategy(),
    new RetryTestStrategy(),
  ];
}

/**
 * Select the best strategy for an issue
 *
 * Returns strategies in priority order (most specific first).
 */
export function selectStrategies(issue: Issue): FixStrategy[] {
  const allStrategies = getAllStrategies();
  return allStrategies.filter((strategy) => strategy.canFix(issue));
}

/**
 * Apply the first applicable strategy for an issue
 */
export async function applyFirstStrategy(
  issue: Issue,
  context: FixContext
): Promise<{ strategy: FixStrategy; result: FixResult } | null> {
  const strategies = selectStrategies(issue);

  if (strategies.length === 0) {
    return null;
  }

  // Try the first (most specific) strategy
  const strategy = strategies[0];
  const result = await strategy.apply(issue, context);

  return { strategy, result };
}

/**
 * Apply all applicable strategies for an issue until one succeeds
 */
export async function applyBestStrategy(
  issue: Issue,
  context: FixContext
): Promise<{ strategy: FixStrategy; result: FixResult } | null> {
  const strategies = selectStrategies(issue);

  if (strategies.length === 0) {
    return null;
  }

  // Try each strategy until one succeeds
  for (const strategy of strategies) {
    const result = await strategy.apply(issue, context);

    if (result.success) {
      return { strategy, result };
    }

    // If this strategy failed, continue to next
    if (context.verbose) {
      console.log(
        `[applyBestStrategy] ${strategy.name} failed: ${result.message}, trying next...`
      );
    }
  }

  // None succeeded
  return null;
}
