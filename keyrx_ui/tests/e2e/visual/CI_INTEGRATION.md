# CI Integration for Visual Regression Tests

This document explains how to integrate visual regression tests into your CI/CD pipeline.

## Quick Setup

Add this job to your `.github/workflows/ci.yml`:

```yaml
visual-regression:
  name: Visual Regression Tests
  runs-on: ubuntu-latest
  timeout-minutes: 20
  needs: build-and-verify

  steps:
    - uses: actions/checkout@v4

    - name: Setup Node.js
      uses: actions/setup-node@v4
      with:
        node-version: '18'
        cache: 'npm'
        cache-dependency-path: keyrx_ui/package-lock.json

    - name: Install frontend dependencies
      run: |
        cd keyrx_ui
        npm ci

    - name: Install Playwright browsers
      run: |
        cd keyrx_ui
        npx playwright install --with-deps chromium

    - name: Build daemon (for E2E tests)
      run: |
        cargo build -p keyrx_daemon --release

    - name: Run visual regression tests
      run: |
        cd keyrx_ui
        npm run test:visual
      env:
        CI: true

    - name: Upload visual diff artifacts
      if: failure()
      uses: actions/upload-artifact@v3
      with:
        name: visual-diffs
        path: |
          keyrx_ui/test-results/
          keyrx_ui/playwright-report/
        retention-days: 30

    - name: Upload baseline snapshots
      if: success()
      uses: actions/upload-artifact@v3
      with:
        name: visual-baselines
        path: |
          keyrx_ui/tests/e2e/visual/*.spec.ts-snapshots/
        retention-days: 7
```

## Baseline Management in CI

### Strategy 1: Commit Baselines to Git (Recommended)

**Pros:**
- Baselines are versioned alongside code
- Easy to review changes in PRs
- No extra CI setup needed

**Cons:**
- Repo size increases (but not significantly for a few screenshots)

**Implementation:**
1. Generate baselines locally on Linux (or in Docker):
   ```bash
   npm run test:visual:update
   ```

2. Commit baselines to git:
   ```bash
   git add tests/e2e/visual/*.spec.ts-snapshots/
   git commit -m "chore: update visual regression baselines"
   ```

3. CI will compare against committed baselines

### Strategy 2: Generate Baselines in CI

**Pros:**
- Guaranteed consistent environment
- No manual baseline generation

**Cons:**
- First run always passes (no baseline to compare)
- Need artifact storage between runs

**Implementation:**
1. Add step to generate baselines on main branch:
   ```yaml
   - name: Generate baselines (main branch only)
     if: github.ref == 'refs/heads/main'
     run: |
       cd keyrx_ui
       npm run test:visual:update

   - name: Upload new baselines
     if: github.ref == 'refs/heads/main'
     uses: actions/upload-artifact@v3
     with:
       name: visual-baselines
       path: keyrx_ui/tests/e2e/visual/*.spec.ts-snapshots/
   ```

2. On PR branches, download baselines from main:
   ```yaml
   - name: Download baselines
     if: github.ref != 'refs/heads/main'
     uses: dawidd6/action-download-artifact@v2
     with:
       workflow: ci.yml
       branch: main
       name: visual-baselines
       path: keyrx_ui/tests/e2e/visual/
   ```

## Handling Visual Failures in CI

When visual tests fail in CI:

1. **Review the Diff**
   - Go to the failed CI run
   - Download the "visual-diffs" artifact
   - Open `playwright-report/index.html` locally
   - Review expected vs. actual screenshots

2. **If Changes Are Intentional**
   - Update baselines locally: `npm run test:visual:update`
   - Commit updated snapshots
   - Push to trigger new CI run

3. **If Changes Are Bugs**
   - Fix the UI issue
   - Push fix to trigger new CI run

## PR Integration

### Show Visual Diffs in PRs

Use GitHub Actions to comment on PRs with visual diff links:

```yaml
- name: Comment PR with visual diffs
  if: failure() && github.event_name == 'pull_request'
  uses: actions/github-script@v6
  with:
    script: |
      const fs = require('fs');
      const comment = `
      ## ⚠️ Visual Regression Failures

      Visual regression tests detected changes. Please review:

      - Download the \`visual-diffs\` artifact from this CI run
      - Open \`playwright-report/index.html\` to see visual differences
      - If intentional, update baselines: \`npm run test:visual:update\`
      - Commit the updated snapshots

      [View CI Run](https://github.com/${{ github.repository }}/actions/runs/${{ github.run_id }})
      `;

      github.rest.issues.createComment({
        issue_number: context.issue.number,
        owner: context.repo.owner,
        repo: context.repo.repo,
        body: comment
      });
```

### Require Visual Approval

Make visual regression a required check:

1. Go to repo Settings → Branches
2. Add branch protection rule for main
3. Enable "Require status checks to pass"
4. Add "Visual Regression Tests" to required checks

## Optimizing CI Performance

### 1. Use Playwright Docker Image

Faster browser installation:

```yaml
runs-on: ubuntu-latest
container:
  image: mcr.microsoft.com/playwright:v1.40.0-focal
```

### 2. Cache Playwright Browsers

```yaml
- name: Cache Playwright browsers
  uses: actions/cache@v3
  with:
    path: ~/.cache/ms-playwright
    key: playwright-${{ runner.os }}-${{ hashFiles('keyrx_ui/package-lock.json') }}
```

### 3. Run Visual Tests in Parallel

If you have many visual tests, split them:

```yaml
strategy:
  matrix:
    shard: [1, 2, 3]
steps:
  - name: Run visual tests (shard ${{ matrix.shard }})
    run: |
      cd keyrx_ui
      npx playwright test tests/e2e/visual --shard=${{ matrix.shard }}/3
```

### 4. Run Only on Visual Changes

Skip visual tests if no UI files changed:

```yaml
- name: Check if visual tests needed
  id: check
  run: |
    if git diff --name-only ${{ github.event.before }} ${{ github.sha }} | grep -E '\.(tsx|css|scss)$'; then
      echo "::set-output name=run::true"
    else
      echo "::set-output name=run::false"
    fi

- name: Run visual tests
  if: steps.check.outputs.run == 'true'
  run: npm run test:visual
```

## Troubleshooting CI Failures

### "No baselines found"

**Cause**: First run, or baselines not committed.

**Fix**: Generate and commit baselines:
```bash
npm run test:visual:update
git add tests/e2e/visual/*.spec.ts-snapshots/
git commit -m "chore: add visual regression baselines"
```

### "Tests pass locally but fail in CI"

**Cause**: Platform differences (fonts, rendering).

**Fix**: Generate baselines on Linux (same as CI):
```bash
# Use Docker to match CI environment
docker run -v $(pwd):/workspace -w /workspace/keyrx_ui \
  mcr.microsoft.com/playwright:v1.40.0-focal \
  sh -c "npm ci && npm run test:visual:update"
```

### "Visual diffs on every run"

**Cause**: Dynamic content (timestamps, animations).

**Fix**: Hide dynamic elements in tests (see `key-pages.spec.ts` examples).

### "Baselines not updating in CI"

**Cause**: Artifact upload step not running.

**Fix**: Check if step runs only on main branch:
```yaml
if: github.ref == 'refs/heads/main'
```

## Best Practices

1. ✅ **Commit baselines to git** - Simplest and most reliable
2. ✅ **Use Linux for baselines** - Match CI environment
3. ✅ **Review diffs before updating** - Never blindly update
4. ✅ **Upload artifacts on failure** - Essential for debugging
5. ✅ **Disable animations** - Configured in tests
6. ✅ **Hide dynamic content** - Timestamps, cursors, etc.
7. ✅ **Use tolerance threshold** - `maxDiffPixelRatio: 0.002`
8. ✅ **Make it a required check** - Prevent visual regressions
9. ✅ **Document baseline updates** - Clear commit messages
10. ✅ **Test locally first** - Don't rely on CI for feedback

## Example: Complete CI Workflow

See `.github/workflows/visual-regression.yml` for a complete example implementation.
