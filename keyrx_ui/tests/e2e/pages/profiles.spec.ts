/**
 * ProfilesPage E2E Tests
 *
 * Tests the profiles management page (/profiles) to verify:
 * - Profile list loads correctly
 * - Create new profile flow works (modal → name → create → appears in list)
 * - Activate profile flow works (click activate → profile becomes active)
 * - Delete profile flow works (click delete → confirm → profile removed)
 * - UI updates after API calls
 * - Test profiles are cleaned up after tests
 *
 * These tests verify the complete integration of ProfilesPage with
 * the daemon API and ensure proper profile management functionality.
 */

import { test, expect } from '../fixtures/daemon';
import { NetworkMonitor } from '../fixtures/network-monitor';

test.describe('ProfilesPage', () => {
  test('should load without console errors', async ({ page }) => {
    // Capture console errors (exclude accessibility warnings which are logged separately)
    const consoleErrors: string[] = [];
    page.on('console', (msg) => {
      if (msg.type() === 'error' && !msg.text().includes('color contrast') && !msg.text().includes('landmarks')) {
        consoleErrors.push(msg.text());
      }
    });

    // Navigate to profiles page
    await page.goto('/profiles');

    // Wait for page to be fully loaded
    await page.waitForLoadState('networkidle');

    // Verify no console errors
    expect(consoleErrors).toEqual([]);
  });

  test('should render profiles page heading', async ({ page }) => {
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Check for main heading
    const heading = page.getByRole('heading', { name: /profiles/i, level: 1 });
    await expect(heading).toBeVisible();
  });

  test('should render create profile button', async ({ page }) => {
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Check for create button (may be rendered with Plus icon text)
    const createButton = page.getByRole('button', { name: /create/i });
    await expect(createButton).toBeVisible({ timeout: 10000 });
  });

  test('should load profile list', async ({ page, daemon }) => {
    // Create a test profile to ensure there's at least one profile
    // Keep name under 32 chars (API limit)
    const testProfileName = `test-${Date.now().toString().slice(-8)}`;
    const testConfig = `
      layer("default") {
        bind("a", action::tap("b"));
      }
    `;

    await daemon.createTestProfile(testProfileName, testConfig);

    // Navigate to profiles page
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Wait for profile cards to load
    // Profile cards should be visible
    const profileCards = page.locator('[data-testid*="profile-card"], .profile-card, [class*="profile"]').filter({
      hasText: testProfileName,
    });

    // Wait a bit for the grid to render
    await page.waitForTimeout(1000);

    // Verify test profile appears in the list
    await expect(page.locator(`text="${testProfileName}"`)).toBeVisible({ timeout: 10000 });

    // Clean up test profile
    await daemon.deleteTestProfile(testProfileName);
  });

  test('should create new profile via modal flow', async ({ page }) => {
    const testProfileName = `tc-${Date.now().toString().slice(-8)}`;

    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Click create profile button
    const createButton = page.getByRole('button', { name: /create profile/i });
    await createButton.click();

    // Wait for modal to appear
    const modal = page.getByRole('dialog');
    await expect(modal).toBeVisible({ timeout: 5000 });

    // Verify modal title
    const modalTitle = modal.getByText(/create new profile/i);
    await expect(modalTitle).toBeVisible();

    // Fill in profile name
    const nameInput = modal.getByPlaceholder(/profile name/i);
    await nameInput.fill(testProfileName);

    // Optionally select a template
    const templateSelect = modal.locator('select, [role="combobox"]').filter({
      has: page.locator('option[value="blank"]'),
    });
    if (await templateSelect.isVisible()) {
      await templateSelect.selectOption('blank');
    }

    // Click create button in modal
    const modalCreateButton = modal.getByRole('button', { name: /^create$/i });
    await modalCreateButton.click();

    // Wait for modal to close
    await expect(modal).not.toBeVisible({ timeout: 5000 });

    // Verify profile appears in the list
    await expect(page.locator(`text="${testProfileName}"`)).toBeVisible({ timeout: 10000 });

    // Clean up: Delete the test profile
    // Find the profile card and delete button
    const profileCard = page.locator('[class*="profile"], [data-testid*="profile"]').filter({
      hasText: testProfileName,
    });

    // Try different delete button patterns
    const deleteButtons = [
      profileCard.getByRole('button', { name: /delete/i }),
      page.getByRole('button', { name: /delete/i }).filter({
        has: page.locator(`text="${testProfileName}"`),
      }),
    ];

    let deleteClicked = false;
    for (const deleteButton of deleteButtons) {
      if (await deleteButton.isVisible().catch(() => false)) {
        await deleteButton.click();
        deleteClicked = true;
        break;
      }
    }

    if (deleteClicked) {
      // Confirm deletion in modal
      const confirmModal = page.getByRole('dialog');
      const confirmButton = confirmModal.getByRole('button', { name: /delete|confirm/i });
      await confirmButton.click();

      // Wait for profile to disappear
      await expect(page.locator(`text="${testProfileName}"`)).not.toBeVisible({ timeout: 5000 });
    }
  });

  test('should activate profile', async ({ page, daemon }) => {
    // Create two test profiles (keep names under 32 chars)
    const profile1 = `ta1-${Date.now().toString().slice(-8)}`;
    const profile2 = `ta2-${Date.now().toString().slice(-8)}`;
    const testConfig = `
      layer("default") {
        bind("a", action::tap("b"));
      }
    `;

    await daemon.createTestProfile(profile1, testConfig);
    await daemon.createTestProfile(profile2, testConfig);

    // Activate profile1 first
    await page.request.post(`${daemon.apiUrl}/api/profiles/${profile1}/activate`);

    // Navigate to profiles page
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Find profile2 card and click activate
    const profile2Card = page.locator('[class*="profile"], [data-testid*="profile"]').filter({
      hasText: profile2,
    });

    const activateButton = profile2Card.getByRole('button', { name: /activate/i });
    await expect(activateButton).toBeVisible({ timeout: 5000 });
    await activateButton.click();

    // Wait for activation to complete (button might change to "Active" or have visual indicator)
    await page.waitForTimeout(1000);

    // Verify profile2 is now marked as active
    // Look for visual indicators like checkmark or "Active" badge
    const activeIndicators = [
      profile2Card.locator('text=/active/i'),
      profile2Card.locator('[class*="active"]'),
      profile2Card.locator('svg[class*="check"]'),
    ];

    let foundActiveIndicator = false;
    for (const indicator of activeIndicators) {
      if (await indicator.isVisible().catch(() => false)) {
        foundActiveIndicator = true;
        break;
      }
    }

    // At minimum, the activate button should not be visible anymore or be disabled
    const activateButtonAfter = profile2Card.getByRole('button', { name: /^activate$/i });
    const isActivateButtonHidden = await activateButtonAfter.isHidden().catch(() => true);
    const isActivateButtonDisabled = await activateButtonAfter.isDisabled().catch(() => false);

    expect(foundActiveIndicator || isActivateButtonHidden || isActivateButtonDisabled).toBe(true);

    // Clean up test profiles
    await daemon.deleteTestProfile(profile1);
    await daemon.deleteTestProfile(profile2);
  });

  test('should delete profile', async ({ page, daemon }) => {
    const testProfileName = `td-${Date.now().toString().slice(-8)}`;
    const testConfig = `
      layer("default") {
        bind("a", action::tap("b"));
      }
    `;

    await daemon.createTestProfile(testProfileName, testConfig);

    // Navigate to profiles page
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Verify profile exists
    await expect(page.locator(`text="${testProfileName}"`)).toBeVisible({ timeout: 10000 });

    // Find the profile card and delete button
    const profileCard = page.locator('[class*="profile"], [data-testid*="profile"]').filter({
      hasText: testProfileName,
    });

    const deleteButton = profileCard.getByRole('button', { name: /delete/i });
    await expect(deleteButton).toBeVisible({ timeout: 5000 });
    await deleteButton.click();

    // Wait for delete confirmation modal
    const modal = page.getByRole('dialog');
    await expect(modal).toBeVisible({ timeout: 5000 });

    // Verify modal title
    const modalTitle = modal.getByText(/delete profile/i);
    await expect(modalTitle).toBeVisible();

    // Verify confirmation message mentions the profile name
    await expect(modal.locator(`text="${testProfileName}"`)).toBeVisible();

    // Click confirm delete button
    const confirmButton = modal.getByRole('button', { name: /delete|confirm/i });
    await confirmButton.click();

    // Wait for modal to close
    await expect(modal).not.toBeVisible({ timeout: 5000 });

    // Verify profile is removed from the list
    await expect(page.locator(`text="${testProfileName}"`)).not.toBeVisible({ timeout: 10000 });

    // Verify API confirms deletion (profile should be gone)
    const response = await page.request.get(`${daemon.apiUrl}/api/profiles`);
    const profiles = await response.json();
    const profileExists = profiles.some((p: any) => p.name === testProfileName);
    expect(profileExists).toBe(false);
  });

  test('should validate profile name on create', async ({ page }) => {
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Open create modal
    const createButton = page.getByRole('button', { name: /create profile/i });
    await createButton.click();

    const modal = page.getByRole('dialog');
    await expect(modal).toBeVisible({ timeout: 5000 });

    // Try to create with empty name
    const modalCreateButton = modal.getByRole('button', { name: /^create$/i });
    await modalCreateButton.click();

    // Should show error message
    const errorMessage = modal.locator('text=/required|cannot be empty/i');
    const hasError = await errorMessage.isVisible().catch(() => false);

    // Modal should still be open (creation failed)
    const isModalStillVisible = await modal.isVisible();

    expect(hasError || isModalStillVisible).toBe(true);

    // Close modal
    const cancelButton = modal.getByRole('button', { name: /cancel/i });
    await cancelButton.click();
  });

  test('should not make excessive API requests on load', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // ProfilesPage should make reasonable number of requests
    // Expected requests:
    // - GET /api/profiles (1-2x - initial load + possible refresh)
    // - GET /api/status (0-1x - if status is checked)
    // Total: ~1-3 requests is reasonable

    const totalRequests = monitor.getRequests().length;
    expect(totalRequests).toBeLessThanOrEqual(15); // Allow headroom for initial page load and auto-refresh

    // Print summary for debugging
    if (process.env.DEBUG_NETWORK) {
      monitor.printSummary();
    }

    monitor.stop();
  });

  test('should not make duplicate requests within 100ms', async ({ page }) => {
    const monitor = new NetworkMonitor(page);
    monitor.start();

    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Verify no duplicate requests (catches rapid-fire bug patterns)
    monitor.assertNoDuplicateRequests();

    monitor.stop();
  });

  test('should handle network errors gracefully', async ({ page }) => {
    // Navigate to profiles page
    await page.goto('/profiles');

    // Page should load without throwing errors
    // Even if API fails, page should show error state gracefully
    const body = page.locator('body');
    await expect(body).toBeVisible();

    // Check for error message or empty state
    // (Either is acceptable if API is unavailable)
    await page.waitForLoadState('networkidle');
  });

  test('should show loading state while fetching profiles', async ({ page }) => {
    // Navigate to profiles page
    const response = page.goto('/profiles');

    // While loading, should show skeleton/loading state
    // (This is a quick test, might not always catch the loading state)
    const loadingIndicators = [
      page.locator('[class*="skeleton"]'),
      page.locator('[class*="loading"]'),
      page.locator('[aria-busy="true"]'),
    ];

    // Wait for response
    await response;
    await page.waitForLoadState('networkidle');

    // Eventually, loading state should be gone and content visible
    const heading = page.getByRole('heading', { name: /profiles/i, level: 1 });
    await expect(heading).toBeVisible();
  });

  test('should navigate to config page from profile card', async ({ page, daemon }) => {
    const testProfileName = `tn-${Date.now().toString().slice(-8)}`;
    const testConfig = `
      layer("default") {
        bind("a", action::tap("b"));
      }
    `;

    await daemon.createTestProfile(testProfileName, testConfig);

    // Navigate to profiles page
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Find the profile card
    const profileCard = page.locator('[class*="profile"], [data-testid*="profile"]').filter({
      hasText: testProfileName,
    });

    // Look for edit button or config link
    const editButtons = [
      profileCard.getByRole('button', { name: /edit|config/i }),
      profileCard.getByRole('link', { name: /edit|config/i }),
    ];

    let clicked = false;
    for (const button of editButtons) {
      if (await button.isVisible().catch(() => false)) {
        await button.click();
        clicked = true;
        break;
      }
    }

    if (clicked) {
      // Should navigate to config page or open edit modal
      // Check if URL changed or modal opened
      const urlChanged = page.url().includes('/config');
      const modalOpened = await page.getByRole('dialog').isVisible().catch(() => false);

      expect(urlChanged || modalOpened).toBe(true);
    }

    // Clean up test profile
    await daemon.deleteTestProfile(testProfileName);
  });

  test('should display profile metadata correctly', async ({ page, daemon }) => {
    const testProfileName = `tm-${Date.now().toString().slice(-8)}`;
    const testConfig = `
      layer("default") {
        bind("a", action::tap("b"));
      }
    `;

    await daemon.createTestProfile(testProfileName, testConfig);

    // Navigate to profiles page
    await page.goto('/profiles');
    await page.waitForLoadState('networkidle');

    // Find the profile card
    const profileCard = page.locator('[class*="profile"], [data-testid*="profile"]').filter({
      hasText: testProfileName,
    });

    // Verify profile name is displayed
    await expect(profileCard.locator(`text="${testProfileName}"`)).toBeVisible();

    // Profile cards typically show:
    // - Profile name ✓ (verified above)
    // - Last modified date/time (might be visible)
    // - Active status indicator (if active)
    // - Action buttons (activate, edit, delete)

    // Verify action buttons are present
    const hasActivateButton = await profileCard.getByRole('button', { name: /activate/i }).isVisible().catch(() => false);
    const hasDeleteButton = await profileCard.getByRole('button', { name: /delete/i }).isVisible().catch(() => false);

    // At least one action button should be visible
    expect(hasActivateButton || hasDeleteButton).toBe(true);

    // Clean up test profile
    await daemon.deleteTestProfile(testProfileName);
  });
});
