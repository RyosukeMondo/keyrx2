/**
 * Type definitions for API test cases
 */

/**
 * Scenario definition from expected-results.json
 */
export interface ScenarioDefinition {
  method: string;
  status: number;
  headers?: Record<string, string>;
  body: unknown;
  requestBody?: unknown;
  pathParams?: Record<string, string>;
  ignoreFields?: string[];
}

/**
 * Expected results database structure
 */
export interface ExpectedResults {
  version: string;
  apiVersion: string;
  description?: string;
  endpoints: Record<string, {
    scenarios: Record<string, ScenarioDefinition>;
  }>;
  commonScenarios?: Record<string, ScenarioDefinition>;
  metadata?: {
    lastUpdated?: string;
    author?: string;
    notes?: string[];
  };
}

/**
 * Test execution context
 */
export interface TestContext {
  /** API client instance */
  client: unknown;
  /** Expected results database */
  expectedResults: ExpectedResults;
  /** Test execution timeout (ms) */
  timeout: number;
  /** Enable verbose logging */
  verbose: boolean;
}

/**
 * Test suite result
 */
export interface TestSuiteResult {
  /** Total number of tests */
  total: number;
  /** Number of passed tests */
  passed: number;
  /** Number of failed tests */
  failed: number;
  /** Number of skipped tests */
  skipped: number;
  /** Total execution duration (ms) */
  duration: number;
  /** Individual test results */
  results: TestExecutionResult[];
  /** Timestamp when suite started */
  startTime: string;
  /** Timestamp when suite finished */
  endTime: string;
}

/**
 * Individual test execution result
 */
export interface TestExecutionResult {
  /** Test case ID */
  id: string;
  /** Test name */
  name: string;
  /** Test endpoint */
  endpoint: string;
  /** Test scenario */
  scenario: string;
  /** Test category */
  category: string;
  /** Test priority */
  priority: number;
  /** Test status */
  status: 'passed' | 'failed' | 'skipped' | 'error';
  /** Execution duration (ms) */
  duration: number;
  /** Error message if failed */
  error?: string;
  /** Actual response received */
  actual?: unknown;
  /** Expected response */
  expected?: unknown;
  /** Diff between actual and expected */
  diff?: unknown;
  /** Stack trace if error */
  stackTrace?: string;
}
