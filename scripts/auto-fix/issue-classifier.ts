/**
 * Issue Classifier for Auto-Fix System
 *
 * Analyzes test failures and classifies them into fixable categories:
 * - Network errors: connectivity, timeouts, daemon issues
 * - Validation errors: schema mismatches, type errors, missing/extra fields
 * - Logic errors: incorrect values, wrong business logic
 * - Data errors: empty arrays, missing fixtures, seed data issues
 *
 * Each issue is assigned:
 * - Type: category of the issue
 * - Fixable: whether automated fix is possible
 * - Priority: 1 (auto-fixable), 2 (needs hint), 3 (manual)
 * - Description: human-readable explanation
 * - Suggested fix: actionable remediation step (if fixable)
 */

import type { TestExecutionResult } from '../test-cases/types.js';
import type { Diff } from '../comparator/response-comparator.js';

/**
 * Issue type classification
 */
export type IssueType = 'network' | 'validation' | 'logic' | 'data';

/**
 * Classified issue with fix suggestions
 */
export interface Issue {
  /** Category of the issue */
  type: IssueType;
  /** Whether this issue can be automatically fixed */
  fixable: boolean;
  /** Priority: 1 (auto-fixable), 2 (needs hint), 3 (manual) */
  priority: 1 | 2 | 3;
  /** Human-readable description */
  description: string;
  /** Suggested fix action (if fixable) */
  suggestedFix?: string;
  /** Original error message */
  originalError?: string;
  /** Diff information (if applicable) */
  diff?: Diff[];
  /** Related test info */
  testId: string;
  testName: string;
  endpoint: string;
}

/**
 * Classification result for a test failure
 */
export interface ClassificationResult {
  /** List of identified issues */
  issues: Issue[];
  /** Summary statistics */
  summary: {
    total: number;
    fixable: number;
    priority1: number;
    priority2: number;
    priority3: number;
  };
}

/**
 * Issue classifier - analyzes test failures and suggests fixes
 */
export class IssueClassifier {
  /**
   * Classify issues from a single failed test result
   */
  classifyTest(testResult: TestExecutionResult): Issue[] {
    const issues: Issue[] = [];

    // Only classify failed or error tests
    if (testResult.status === 'passed' || testResult.status === 'skipped') {
      return issues;
    }

    // Classify based on error type
    if (testResult.status === 'error') {
      // Test execution error (not assertion failure)
      const issue = this.classifyExecutionError(testResult);
      if (issue) {
        issues.push(issue);
      }
    } else if (testResult.status === 'failed') {
      // Assertion failure - analyze diff
      const diffIssues = this.classifyAssertionFailure(testResult);
      issues.push(...diffIssues);
    }

    return issues;
  }

  /**
   * Classify issues from multiple test results
   */
  classifyAll(testResults: TestExecutionResult[]): ClassificationResult {
    const allIssues: Issue[] = [];

    for (const result of testResults) {
      const issues = this.classifyTest(result);
      allIssues.push(...issues);
    }

    // Calculate summary
    const summary = {
      total: allIssues.length,
      fixable: allIssues.filter((i) => i.fixable).length,
      priority1: allIssues.filter((i) => i.priority === 1).length,
      priority2: allIssues.filter((i) => i.priority === 2).length,
      priority3: allIssues.filter((i) => i.priority === 3).length,
    };

    return {
      issues: allIssues,
      summary,
    };
  }

  /**
   * Classify execution errors (network, timeout, etc.)
   */
  private classifyExecutionError(testResult: TestExecutionResult): Issue | null {
    const error = testResult.error || '';
    const testInfo = {
      testId: testResult.id,
      testName: testResult.name,
      endpoint: testResult.endpoint,
      originalError: error,
    };

    // Network errors
    if (this.isNetworkError(error)) {
      return {
        ...testInfo,
        type: 'network',
        fixable: true,
        priority: 1,
        description: 'Network connection error - daemon may not be running or unreachable',
        suggestedFix: 'Restart daemon and retry test',
      };
    }

    // Timeout errors
    if (this.isTimeoutError(error)) {
      return {
        ...testInfo,
        type: 'network',
        fixable: true,
        priority: 1,
        description: 'Request timeout - daemon may be slow or unresponsive',
        suggestedFix: 'Wait longer for daemon to start, then retry test',
      };
    }

    // HTTP errors (4xx, 5xx)
    const httpStatus = this.extractHttpStatus(error);
    if (httpStatus !== null) {
      if (httpStatus >= 500) {
        return {
          ...testInfo,
          type: 'network',
          fixable: true,
          priority: 1,
          description: `Server error (${httpStatus}) - daemon internal error`,
          suggestedFix: 'Restart daemon and retry test',
        };
      } else if (httpStatus >= 400) {
        return {
          ...testInfo,
          type: 'logic',
          fixable: false,
          priority: 3,
          description: `Client error (${httpStatus}) - invalid request or endpoint not found`,
          suggestedFix: 'Check API contract and test case definition',
        };
      }
    }

    // Unknown execution error
    return {
      ...testInfo,
      type: 'logic',
      fixable: false,
      priority: 3,
      description: 'Unknown execution error',
      suggestedFix: 'Inspect error message and stack trace manually',
    };
  }

  /**
   * Classify assertion failures based on diff analysis
   */
  private classifyAssertionFailure(testResult: TestExecutionResult): Issue[] {
    const issues: Issue[] = [];
    const testInfo = {
      testId: testResult.id,
      testName: testResult.name,
      endpoint: testResult.endpoint,
      originalError: testResult.error,
    };

    // If no diff available, treat as general assertion failure
    if (!testResult.diff || !Array.isArray(testResult.diff)) {
      issues.push({
        ...testInfo,
        type: 'logic',
        fixable: false,
        priority: 3,
        description: 'Assertion failure without detailed diff information',
        suggestedFix: 'Enable diff output in test executor',
      });
      return issues;
    }

    const diffs = testResult.diff as Diff[];

    // Analyze each diff
    for (const diff of diffs) {
      const issue = this.classifyDiff(diff, testInfo);
      if (issue) {
        issues.push(issue);
      }
    }

    // If no issues classified, add generic failure
    if (issues.length === 0) {
      issues.push({
        ...testInfo,
        type: 'logic',
        fixable: false,
        priority: 3,
        description: 'Assertion failure with unclassified diffs',
        diff: diffs,
      });
    }

    return issues;
  }

  /**
   * Classify individual diff
   */
  private classifyDiff(
    diff: Diff,
    testInfo: { testId: string; testName: string; endpoint: string; originalError?: string }
  ): Issue | null {
    // Type mismatches - likely schema issue
    if (diff.type === 'type-mismatch') {
      return {
        ...testInfo,
        type: 'validation',
        fixable: true,
        priority: 2,
        description: `Type mismatch at ${diff.path}: expected ${this.describeValue(diff.expected)} but got ${this.describeValue(diff.actual)}`,
        suggestedFix: 'Update expected results schema or fix API response type',
        diff: [diff],
      };
    }

    // Missing fields - could be schema change or API bug
    if (diff.type === 'missing') {
      // Empty collections are data issues, not validation
      if (this.isEmptyCollection(diff.expected)) {
        return {
          ...testInfo,
          type: 'data',
          fixable: true,
          priority: 2,
          description: `Missing data at ${diff.path}: expected non-empty but got empty or null`,
          suggestedFix: 'Re-seed fixtures or update expected results for empty case',
          diff: [diff],
        };
      }

      return {
        ...testInfo,
        type: 'validation',
        fixable: true,
        priority: 2,
        description: `Missing field at ${diff.path}`,
        suggestedFix: 'Update expected results to remove field or fix API to include it',
        diff: [diff],
      };
    }

    // Extra fields - likely schema change
    if (diff.type === 'extra') {
      // Check if it's a timestamp or generated ID (commonly ignored)
      if (this.isIgnorableField(diff.path)) {
        return {
          ...testInfo,
          type: 'validation',
          fixable: true,
          priority: 1,
          description: `Extra field at ${diff.path} (likely timestamp or ID)`,
          suggestedFix: 'Add field to ignore list in test configuration',
          diff: [diff],
        };
      }

      return {
        ...testInfo,
        type: 'validation',
        fixable: true,
        priority: 2,
        description: `Extra field at ${diff.path}`,
        suggestedFix: 'Update expected results to include field or fix API',
        diff: [diff],
      };
    }

    // Value mismatches - could be logic error or stale expected results
    if (diff.type === 'value-mismatch') {
      // Check if it's empty vs populated (data issue)
      if (this.isEmptyVsPopulated(diff.expected, diff.actual)) {
        return {
          ...testInfo,
          type: 'data',
          fixable: true,
          priority: 2,
          description: `Data mismatch at ${diff.path}: expected populated but got empty (or vice versa)`,
          suggestedFix: 'Re-seed fixtures with appropriate test data',
          diff: [diff],
        };
      }

      // Check if values are similar (typo or minor change)
      if (this.areSimilarValues(diff.expected, diff.actual)) {
        return {
          ...testInfo,
          type: 'validation',
          fixable: true,
          priority: 2,
          description: `Similar but not equal values at ${diff.path}`,
          suggestedFix: 'Update expected results with new value if change is intentional',
          diff: [diff],
        };
      }

      // General value mismatch - likely logic error
      return {
        ...testInfo,
        type: 'logic',
        fixable: false,
        priority: 3,
        description: `Value mismatch at ${diff.path}`,
        suggestedFix: 'Verify business logic and expected behavior',
        diff: [diff],
      };
    }

    return null;
  }

  /**
   * Check if error is a network connectivity error
   */
  private isNetworkError(error: string): boolean {
    const networkPatterns = [
      /ECONNREFUSED/i,
      /ENOTFOUND/i,
      /ECONNRESET/i,
      /ETIMEDOUT/i,
      /network error/i,
      /connection refused/i,
      /connection reset/i,
      /socket hang up/i,
      /fetch failed/i,
    ];
    return networkPatterns.some((pattern) => pattern.test(error));
  }

  /**
   * Check if error is a timeout error
   */
  private isTimeoutError(error: string): boolean {
    const timeoutPatterns = [
      /timeout/i,
      /timed out/i,
      /ETIMEDOUT/i,
      /exceeded/i,
    ];
    return timeoutPatterns.some((pattern) => pattern.test(error));
  }

  /**
   * Extract HTTP status code from error message
   */
  private extractHttpStatus(error: string): number | null {
    const statusMatch = error.match(/status\s+(?:code\s+)?(\d{3})/i);
    if (statusMatch) {
      return parseInt(statusMatch[1], 10);
    }

    // Check for common status phrases
    const statusPhrases: Record<string, number> = {
      'bad request': 400,
      'unauthorized': 401,
      'forbidden': 403,
      'not found': 404,
      'method not allowed': 405,
      'conflict': 409,
      'internal server error': 500,
      'bad gateway': 502,
      'service unavailable': 503,
      'gateway timeout': 504,
    };

    const lowerError = error.toLowerCase();
    for (const [phrase, status] of Object.entries(statusPhrases)) {
      if (lowerError.includes(phrase)) {
        return status;
      }
    }

    return null;
  }

  /**
   * Check if a value is an empty collection
   */
  private isEmptyCollection(value: unknown): boolean {
    if (Array.isArray(value)) {
      return value.length === 0;
    }
    if (value && typeof value === 'object') {
      return Object.keys(value).length === 0;
    }
    return false;
  }

  /**
   * Check if one value is empty and the other is populated
   */
  private isEmptyVsPopulated(value1: unknown, value2: unknown): boolean {
    const isEmpty1 = this.isEmptyCollection(value1) || value1 === null || value1 === undefined;
    const isEmpty2 = this.isEmptyCollection(value2) || value2 === null || value2 === undefined;
    return isEmpty1 !== isEmpty2;
  }

  /**
   * Check if field name suggests it should be ignored (timestamp, id, etc.)
   */
  private isIgnorableField(path: string): boolean {
    const ignorablePatterns = [
      /timestamp/i,
      /\.id$/i,
      /\.uuid$/i,
      /\.created_?at/i,
      /\.updated_?at/i,
      /\.modified_?at/i,
      /\.generated_?at/i,
      /\.date/i,
      /\.time/i,
    ];
    return ignorablePatterns.some((pattern) => pattern.test(path));
  }

  /**
   * Check if two values are similar (for typo/minor change detection)
   */
  private areSimilarValues(value1: unknown, value2: unknown): boolean {
    // Only compare strings
    if (typeof value1 !== 'string' || typeof value2 !== 'string') {
      return false;
    }

    // Case-insensitive match
    if (value1.toLowerCase() === value2.toLowerCase()) {
      return true;
    }

    // Levenshtein-like similarity (simple version)
    const len1 = value1.length;
    const len2 = value2.length;
    const maxLen = Math.max(len1, len2);

    if (maxLen === 0) return true;
    if (maxLen > 100) return false; // Skip large strings

    // Simple similarity: count matching characters
    const commonChars = this.countCommonCharacters(value1.toLowerCase(), value2.toLowerCase());
    const similarity = commonChars / maxLen;

    return similarity >= 0.8; // 80% similarity threshold
  }

  /**
   * Count common characters between two strings
   */
  private countCommonCharacters(str1: string, str2: string): number {
    const chars1 = str1.split('');
    const chars2 = str2.split('');
    let common = 0;

    for (const char of chars1) {
      const index = chars2.indexOf(char);
      if (index !== -1) {
        common++;
        chars2.splice(index, 1); // Remove to avoid double-counting
      }
    }

    return common;
  }

  /**
   * Describe a value for human-readable output
   */
  private describeValue(value: unknown): string {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value === 'string') return `string("${value}")`;
    if (typeof value === 'number') return `number(${value})`;
    if (typeof value === 'boolean') return `boolean(${value})`;
    if (Array.isArray(value)) return `array[${value.length}]`;
    if (typeof value === 'object') return 'object';
    return String(value);
  }
}

/**
 * Create an issue classifier instance
 */
export function createIssueClassifier(): IssueClassifier {
  return new IssueClassifier();
}

/**
 * Classify a single test result
 *
 * @example
 * const issues = classifyTestResult(failedTest);
 * console.log(`Found ${issues.length} issues`);
 */
export function classifyTestResult(testResult: TestExecutionResult): Issue[] {
  const classifier = createIssueClassifier();
  return classifier.classifyTest(testResult);
}

/**
 * Classify multiple test results
 *
 * @example
 * const classification = classifyTestResults(results.results.filter(r => r.status !== 'passed'));
 * console.log(`${classification.summary.fixable}/${classification.summary.total} issues are fixable`);
 */
export function classifyTestResults(testResults: TestExecutionResult[]): ClassificationResult {
  const classifier = createIssueClassifier();
  return classifier.classifyAll(testResults);
}
