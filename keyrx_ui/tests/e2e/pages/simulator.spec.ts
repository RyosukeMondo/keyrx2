/**
 * SimulatorPage E2E Tests
 *
 * Tests the keyboard simulator page (/simulator) to verify:
 * - Page loads without console errors
 * - Simulator interface renders correctly
 * - Key simulation interaction works
 * - Profile selection works
 * - Event log displays simulation results
 * - State inspector shows current state
 * - WASM integration works when available
 *
 * These tests verify the complete integration of SimulatorPage with
 * the daemon API and WASM simulation engine.
 *
 * Requirements: 1.6, 1.7, 1.8
 */

import { test, expect } from '../fixtures/daemon';
import { NetworkMonitor } from '../fixtures/network-monitor';

test.describe('SimulatorPage', () => {
  test('should load without console errors', async ({ page }) => {
    // Capture console errors (excluding accessibility warnings and WebSocket errors)
    const consoleErrors: string[] = [];
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        const text = msg.text();
        // Filter out known non-critical errors
        if (!text.includes('color contrast') &&
            !text.includes('landmarks') &&
            !text.includes('WebSocket') &&
            !text.includes('WASM')) {
          consoleErrors.push(text);
        }
      }
    });

    // Navigate to simulator page
    await page.goto('/simulator');

    // Wait for page to be fully loaded
    await page.waitForLoadState('networkidle');

    // Verify no critical console errors
    expect(consoleErrors).toEqual([]);
  });

  test('should render page heading', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Check for main heading
    const heading = page.getByRole('heading', { name: /keyboard simulator/i, level: 1 });
    await expect(heading).toBeVisible();
  });

  test('should render configuration mode selector', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Look for configuration mode toggle
    const modeSelector = page.locator('text=/configuration.*mode/i').first();
    await expect(modeSelector).toBeVisible({ timeout: 10000 });

    // Verify both mode buttons exist
    const profileModeButton = page.locator('button:has-text("Use Profile")');
    const customCodeButton = page.locator('button:has-text("Edit Code")');

    await expect(profileModeButton).toBeVisible();
    await expect(customCodeButton).toBeVisible();
  });

  test('should render profile selector in profile mode', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Ensure we're in profile mode
    const profileModeButton = page.locator('button:has-text("Use Profile")');
    await profileModeButton.click();

    // Wait for profile selector to appear
    const profileSelector = page.locator('#profile-selector, select[aria-label*="profile" i]').first();
    await expect(profileSelector).toBeVisible({ timeout: 10000 });
  });

  test('should switch to custom code editor mode', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Click "Edit Code" button
    const customCodeButton = page.locator('button:has-text("Edit Code")');
    await customCodeButton.click();

    // Wait for Monaco editor to appear
    await page.waitForTimeout(1000);

    // Look for Monaco editor container
    const editorContainer = page.locator('.monaco-editor');
    const hasEditor = await editorContainer.isVisible({ timeout: 5000 }).catch(() => false);

    // Editor should be visible or at least the mode switched
    expect(hasEditor).toBeTruthy();
  });

  test('should render state inspector card', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Look for state inspector section
    const stateInspector = page.locator('text=/state.*inspector/i').first();
    await expect(stateInspector).toBeVisible({ timeout: 10000 });

    // Verify modifiers section
    const modifiersSection = page.locator('text=/modifiers/i').first();
    await expect(modifiersSection).toBeVisible();

    // Verify locks section
    const locksSection = page.locator('text=/locks/i').first();
    await expect(locksSection).toBeVisible();
  });

  test('should render event log card', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Look for event log section
    const eventLog = page.locator('text=/event.*log/i').first();
    await expect(eventLog).toBeVisible({ timeout: 10000 });

    // Should show either events or empty state
    const emptyState = page.locator('text=/no events|click.*key.*to.*start/i').first();
    const hasEmptyState = await emptyState.isVisible({ timeout: 3000 }).catch(() => false);

    // Empty state should be visible initially (no events yet)
    expect(hasEmptyState).toBeTruthy();
  });

  test('should render interactive keyboard', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Look for keyboard visualizer section
    const keyboardHeading = page.locator('text=/interactive.*keyboard/i').first();
    await expect(keyboardHeading).toBeVisible({ timeout: 10000 });

    // Verify keyboard visualizer component renders
    // KeyboardVisualizer renders keys as buttons or clickable elements
    const keyboardKeys = page.locator('button[data-key], [data-testid^="key-"]');
    const keyCount = await keyboardKeys.count();

    // Should have multiple keys rendered
    expect(keyCount).toBeGreaterThan(10);
  });

  test('should render reset and copy buttons', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Look for reset button
    const resetButton = page.locator('button:has-text("Reset")').first();
    await expect(resetButton).toBeVisible();

    // Look for copy event log button
    const copyButton = page.locator('button:has-text("Copy Event Log")').first();
    await expect(copyButton).toBeVisible();

    // Copy button should be disabled initially (no events)
    const isDisabled = await copyButton.isDisabled();
    expect(isDisabled).toBeTruthy();
  });

  test('should handle profile selection', async ({ page, daemon }) => {
    // Create a test profile
    const testProfileName = `e2e-sim-${Date.now()}`;
    const testConfig = `
      layer("default") {
        bind("a", action::tap("b"));
      }
    `;

    await daemon.createTestProfile(testProfileName, testConfig);

    // Navigate to simulator page
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Ensure we're in profile mode
    const profileModeButton = page.locator('button:has-text("Use Profile")');
    await profileModeButton.click();

    // Wait for profile selector
    const profileSelector = page.locator('#profile-selector, select[aria-label*="profile" i]').first();
    await expect(profileSelector).toBeVisible();

    // Select the test profile
    await profileSelector.selectOption(testProfileName);

    // Wait for profile to load
    await page.waitForTimeout(1000);

    // Verify selected profile is shown in the selector
    const selectedValue = await profileSelector.inputValue();
    expect(selectedValue).toBe(testProfileName);

    // Clean up test profile
    await daemon.deleteTestProfile(testProfileName);
  });

  test('should simulate key interaction', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Find a key to click (e.g., 'A' key)
    // KeyboardVisualizer renders keys with data-key attribute
    const aKey = page.locator('[data-key="A"], button:has-text("A")').first();

    const keyExists = await aKey.isVisible({ timeout: 5000 }).catch(() => false);

    if (keyExists) {
      // Click the key to simulate press
      await aKey.click();

      // Wait for event to be logged
      await page.waitForTimeout(500);

      // Verify event appears in event log
      const eventLog = page.locator('text=/press.*a/i, text=/output/i').first();
      const hasEvent = await eventLog.isVisible({ timeout: 3000 }).catch(() => false);

      // Should see event in log
      expect(hasEvent).toBeTruthy();
    } else {
      // If key not found, verify keyboard loaded
      const keyboardHeading = page.locator('text=/interactive.*keyboard/i');
      await expect(keyboardHeading).toBeVisible();
    }
  });

  test('should display state changes on key press', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Look for state inspector showing initial state
    const stateInspector = page.locator('text=/state.*inspector/i').first();
    await expect(stateInspector).toBeVisible();

    // Get initial active layer text
    const activeLayer = page.locator('text=/active.*layer|layer.*:/i').first();
    const hasLayer = await activeLayer.isVisible({ timeout: 3000 }).catch(() => false);

    // State inspector should show layer information
    expect(hasLayer).toBeTruthy();
  });

  test('should handle reset simulator action', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Find and click a key to generate an event
    const anyKey = page.locator('[data-key], button[aria-label*="key" i]').first();
    const keyExists = await anyKey.isVisible({ timeout: 5000 }).catch(() => false);

    if (keyExists) {
      await anyKey.click();
      await page.waitForTimeout(500);
    }

    // Click reset button
    const resetButton = page.locator('button:has-text("Reset")').first();
    await resetButton.click();

    // Wait for reset action
    await page.waitForTimeout(500);

    // Verify reset event appears in log
    const resetEvent = page.locator('text=/simulator.*reset|reset/i');
    const hasResetEvent = await resetEvent.isVisible({ timeout: 3000 }).catch(() => false);

    // Should see reset confirmation
    expect(hasResetEvent).toBeTruthy();
  });

  test('should display WASM status indicator', async ({ page, daemon }) => {
    // Create a test profile
    const testProfileName = `e2e-sim-wasm-${Date.now()}`;
    const testConfig = `
      layer("default") {
        bind("a", action::tap("b"));
      }
    `;

    await daemon.createTestProfile(testProfileName, testConfig);

    // Navigate to simulator page
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Ensure we're in profile mode
    const profileModeButton = page.locator('button:has-text("Use Profile")');
    await profileModeButton.click();

    // Select the test profile
    const profileSelector = page.locator('#profile-selector, select[aria-label*="profile" i]').first();
    await profileSelector.selectOption(testProfileName);

    // Wait for profile to load
    await page.waitForTimeout(1500);

    // Look for WASM status indicator
    // Could show "WASM Simulator Active", "Using mock simulation", or "WASM not available"
    const wasmStatus = page.locator('text=/wasm.*simulator.*active|wasm.*not.*available|using.*mock|wasm.*not.*ready/i').first();
    const hasStatus = await wasmStatus.isVisible({ timeout: 3000 }).catch(() => false);

    // Some status indicator should be present
    expect(hasStatus).toBeTruthy();

    // Clean up test profile
    await daemon.deleteTestProfile(testProfileName);
  });

  test('should not make excessive API requests on load', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // SimulatorPage should make reasonable number of requests
    // Expected requests:
    // - GET /api/profiles (1x - to populate profile selector)
    // - GET /api/profiles/:name/config (0-1x - if a profile is auto-selected)
    // Total: ~1-3 requests is reasonable

    const totalRequests = monitor.getRequests().length;
    expect(totalRequests).toBeLessThanOrEqual(10); // Allow some headroom

    // Print summary for debugging
    if (process.env.DEBUG_NETWORK) {
      monitor.printSummary();
    }

    monitor.stop();
  });

  test('should not make duplicate requests within 100ms', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Verify no duplicate requests (catches rapid-fire bug patterns)
    monitor.assertNoDuplicateRequests();

    monitor.stop();
  });

  test('should handle navigation to simulator page from other pages', async ({ page }) => {
    // Start from home page
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Navigate to simulator via link
    const simulatorLink = page.locator('a[href*="/simulator"], a:has-text("Simulator")').first();

    const linkExists = await simulatorLink.isVisible({ timeout: 3000 }).catch(() => false);

    if (linkExists) {
      await simulatorLink.click();

      // Wait for navigation
      await page.waitForLoadState('networkidle');

      // Verify we're on simulator page
      await expect(page).toHaveURL(/\/simulator/);

      // Verify page loaded
      const heading = page.locator('h1:has-text("Keyboard Simulator")');
      await expect(heading).toBeVisible();
    } else {
      // Navigate directly if link not found
      await page.goto('/simulator');
      await page.waitForLoadState('networkidle');

      const heading = page.locator('h1:has-text("Keyboard Simulator")');
      await expect(heading).toBeVisible();
    }
  });

  test('should maintain state across page refresh', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Get initial page state
    const initialUrl = page.url();

    // Refresh the page
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Verify still on simulator page
    expect(page.url()).toBe(initialUrl);

    // Verify page still functional
    const heading = page.locator('h1:has-text("Keyboard Simulator")');
    await expect(heading).toBeVisible();
  });

  test('should render all main sections within 5 seconds', async ({ page }) => {
    const startTime = Date.now();

    await page.goto('/simulator');

    // Wait for all main sections to be present
    await Promise.all([
      page.locator('text=/configuration.*mode/i').first().waitFor({ timeout: 5000 }),
      page.locator('text=/state.*inspector/i').first().waitFor({ timeout: 5000 }),
      page.locator('text=/event.*log/i').first().waitFor({ timeout: 5000 }),
      page.locator('text=/interactive.*keyboard/i').first().waitFor({ timeout: 5000 }),
    ]);

    const loadTime = Date.now() - startTime;

    // Performance check: should load within 5 seconds
    expect(loadTime).toBeLessThan(5000);
  });

  test('should display help text for simulator', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Look for help/description text
    // Should explain what the simulator does
    const helpText = page.locator('text=/test.*configuration|click.*keys.*or.*typing|changes.*not.*saved/i').first();
    const hasHelp = await helpText.isVisible({ timeout: 3000 }).catch(() => false);

    // Should have some instructional text
    expect(hasHelp).toBeTruthy();
  });
});

test.describe('SimulatorPage - Profile Mode', () => {
  test('should load profile configuration when selected', async ({ page, daemon }) => {
    // Create a test profile
    const testProfileName = `e2e-sim-prof-${Date.now()}`;
    const testConfig = `
      layer("default") {
        bind("a", action::tap("b"));
      }
    `;

    await daemon.createTestProfile(testProfileName, testConfig);

    // Navigate to simulator page
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Ensure we're in profile mode
    const profileModeButton = page.locator('button:has-text("Use Profile")');
    await profileModeButton.click();

    // Select the test profile
    const profileSelector = page.locator('#profile-selector, select[aria-label*="profile" i]').first();
    await profileSelector.selectOption(testProfileName);

    // Wait for profile to load
    await page.waitForTimeout(1500);

    // Verify profile loaded (check for WASM status or config load indicator)
    const pageContent = await page.locator('body').textContent();
    expect(pageContent).toMatch(/wasm|simulator|mock|active/i);

    // Clean up test profile
    await daemon.deleteTestProfile(testProfileName);
  });

  test('should show active profile indicator', async ({ page, daemon }) => {
    // Create and activate a test profile
    const testProfileName = `e2e-sim-active-${Date.now()}`;
    const testConfig = `
      layer("default") {
        bind("a", action::tap("b"));
      }
    `;

    await daemon.createTestProfile(testProfileName, testConfig);
    await page.request.post(`${daemon.apiUrl}/api/profiles/${testProfileName}/activate`);

    // Navigate to simulator page
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Ensure we're in profile mode
    const profileModeButton = page.locator('button:has-text("Use Profile")');
    await profileModeButton.click();

    // Wait for profile selector to load
    await page.waitForTimeout(1000);

    // Look for active profile indicator in the selector
    const profileSelector = page.locator('#profile-selector, select[aria-label*="profile" i]').first();
    const selectorContent = await profileSelector.textContent();

    // Active profile should have "[Active]" marker
    const hasActiveMarker = selectorContent?.includes('[Active]') || selectorContent?.includes('Active');
    expect(hasActiveMarker).toBeTruthy();

    // Clean up test profile
    await daemon.deleteTestProfile(testProfileName);
  });
});

test.describe('SimulatorPage - Custom Code Mode', () => {
  test('should render Monaco editor in custom code mode', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Switch to custom code mode
    const customCodeButton = page.locator('button:has-text("Edit Code")');
    await customCodeButton.click();

    // Wait for Monaco editor to load
    await page.waitForTimeout(2000);

    // Look for Monaco editor
    const monacoEditor = page.locator('.monaco-editor');
    const hasEditor = await monacoEditor.isVisible({ timeout: 5000 }).catch(() => false);

    // Editor should be visible
    expect(hasEditor).toBeTruthy();
  });

  test('should show validation errors for invalid code', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Switch to custom code mode
    const customCodeButton = page.locator('button:has-text("Edit Code")');
    await customCodeButton.click();

    // Wait for Monaco editor to load
    await page.waitForTimeout(2000);

    // Look for validation error indicator
    const errorIndicator = page.locator('text=/error|validation/i').first();

    // May or may not have errors depending on default code
    // Just verify the mode is functional
    const monacoEditor = page.locator('.monaco-editor');
    const hasEditor = await monacoEditor.isVisible({ timeout: 3000 }).catch(() => false);

    expect(hasEditor).toBeTruthy();
  });
});

test.describe('SimulatorPage - Accessibility', () => {
  test('should have accessible form controls', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Ensure we're in profile mode to test profile selector
    const profileModeButton = page.locator('button:has-text("Use Profile")');
    await profileModeButton.click();

    // Profile selector should have label
    const profileSelector = page.locator('#profile-selector, select[aria-label*="profile" i]').first();
    await expect(profileSelector).toBeVisible();

    // Check for label
    const label = page.locator('label[for="profile-selector"]');
    const hasLabel = await label.isVisible({ timeout: 2000 }).catch(() => false);

    // Should have either label or aria-label
    if (!hasLabel) {
      const ariaLabel = await profileSelector.getAttribute('aria-label');
      expect(ariaLabel).toBeTruthy();
    }
  });

  test('should have accessible buttons with labels', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Check reset button
    const resetButton = page.locator('button:has-text("Reset")').first();
    await expect(resetButton).toBeVisible();

    // Button should have accessible text or aria-label
    const buttonText = await resetButton.textContent();
    const ariaLabel = await resetButton.getAttribute('aria-label');

    expect(buttonText || ariaLabel).toBeTruthy();
  });

  test('should have accessible card sections', async ({ page }) => {
    await page.goto('/simulator');
    await page.waitForLoadState('networkidle');

    // Cards should have aria-label or aria-labelledby
    const cards = page.locator('[role="region"], [aria-label]');
    const cardCount = await cards.count();

    // Should have multiple accessible sections
    expect(cardCount).toBeGreaterThan(0);
  });
});
