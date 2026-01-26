# Version Management - Single Source of Truth (SSOT)

KeyRx uses a unified version management system to ensure consistency across all components.

## Version Sources

### Primary Source (SSOT)
**`Cargo.toml`** (workspace version)
```toml
[workspace.package]
version = "0.1.0"  # ← SINGLE SOURCE OF TRUTH
```

This version is automatically propagated to:
- All Rust crates (via `workspace = true`)
- UI package.json (via sync script)
- Generated version.ts (via build script)
- System tray application
- Release packages

## How It Works

### 1. Cargo Ecosystem (Automatic)
Rust crates use `env!("CARGO_PKG_VERSION")` to get version at compile time:

```rust
// keyrx_daemon/src/web/api/metrics.rs
version: env!("CARGO_PKG_VERSION").to_string()
```

This automatically reads from Cargo.toml, no manual sync needed.

### 2. UI Ecosystem (Generated)

**Before build**, `scripts/generate-version.js` creates `keyrx_ui/src/version.ts`:

```typescript
// Auto-generated - DO NOT EDIT
export const VERSION = '0.1.0';
export const BUILD_TIME = '2026-01-25T14:01:04.449Z';
export const GIT_COMMIT = '71fb6e7f';
export const GIT_BRANCH = 'main';
```

This provides:
- **VERSION**: From package.json
- **BUILD_TIME**: Actual build timestamp (not runtime)
- **GIT_COMMIT**: Short commit hash
- **GIT_BRANCH**: Current branch

The UI displays this in Layout.tsx:
```tsx
v{VERSION} • {new Date(BUILD_TIME).toLocaleString()}
```

## Synchronization Workflow

### Method 1: Manual Version Bump (Recommended)

When creating a new release:

```bash
# 1. Update SSOT (Cargo.toml)
sed -i 's/version = "0.1.0"/version = "0.2.0"/' Cargo.toml

# 2. Sync to all files
./scripts/sync-version.sh

# 3. Verify
grep -r "0.2.0" Cargo.toml keyrx_ui/package.json keyrx_ui/src/version.ts

# 4. Commit
git add Cargo.toml keyrx_ui/package.json keyrx_ui/src/version.ts
git commit -m "chore: bump version to 0.2.0"
```

### Method 2: Makefile Helper

```bash
# Interactive version bump
make release

# Or update version in Cargo.toml manually, then:
make sync-version
```

### Method 3: Direct Package.json (UI only)

For UI-only changes:

```bash
cd keyrx_ui
npm run sync-version  # Syncs from parent Cargo.toml
npm run build         # Automatically generates version.ts
```

## Build Integration

### UI Builds
The `prebuild` script automatically generates version.ts:

```json
{
  "scripts": {
    "prebuild": "node ../scripts/generate-version.js",
    "build": "npm run build:wasm && tsc -b && vite build"
  }
}
```

### Development Mode
Version is regenerated on dev server start:

```bash
cd keyrx_ui
npm run dev  # Generates version.ts before starting Vite
```

### Release Builds
GitHub Actions automatically syncs versions:

```yaml
- name: Sync Versions
  run: ./scripts/sync-version.sh

- name: Build UI
  run: cd keyrx_ui && npm run build
```

## Version Display Locations

### 1. Web UI Footer
**Location**: `keyrx_ui/src/components/Layout.tsx`
```
v0.1.0 • 1/25/2026, 2:01:04 PM
```

Shows: Version + Build time (localized)

### 2. Daemon API
**Endpoint**: `GET /api/status`
```json
{
  "version": "0.1.0",
  "running": true,
  "profile": "default"
}
```

### 3. System Tray (When Implemented)
About dialog shows version from daemon API.

### 4. Package Metadata
- Debian package: `keyrx_0.2.0_amd64.deb`
- Tarball: `keyrx-0.2.0-linux-x86_64.tar.gz`
- GitHub release: `v0.2.0`

## Verification

### Check All Versions
```bash
# Cargo.toml
grep '^version' Cargo.toml | head -1

# package.json
grep '"version"' keyrx_ui/package.json

# version.ts
grep 'VERSION' keyrx_ui/src/version.ts

# Should all match!
```

### Automated Verification
```bash
./scripts/sync-version.sh  # Shows mismatches if any
```

## Common Issues

### Issue: Version Mismatch
**Symptoms**: Different versions in Cargo.toml and package.json

**Fix**:
```bash
./scripts/sync-version.sh
```

### Issue: BUILD_TIME Shows Runtime Instead of Build Time
**Symptoms**: Build time changes on every page refresh

**Fix**: Ensure `generate-version.js` runs before build:
```bash
cd keyrx_ui
npm run prebuild  # Manually regenerate
npm run build     # Builds with correct timestamp
```

### Issue: Git Commit Shows "unknown"
**Symptoms**: `GIT_COMMIT = 'unknown'` in version.ts

**Fix**: Ensure you're in a git repository:
```bash
git status  # Should work
./scripts/generate-version.js  # Retry
```

## Best Practices

1. **Always update Cargo.toml first** - It's the SSOT
2. **Run sync-version.sh before releases** - Ensures consistency
3. **Commit version.ts** - It's auto-generated but should be tracked
4. **Use semantic versioning** - MAJOR.MINOR.PATCH (e.g., 0.2.0)
5. **Tag releases properly** - Use `v` prefix: `v0.2.0`

## Scripts Reference

| Script | Purpose | When to Use |
|--------|---------|-------------|
| `generate-version.js` | Generate version.ts | Automatically before UI builds |
| `sync-version.sh` | Sync all versions from Cargo.toml | Before releases, after version bump |

## Future Enhancements

- [ ] Auto-detect version bumps in CI/CD
- [ ] Semantic version validation (no breaking changes in patch)
- [ ] Changelog generation from git commits
- [ ] Version compatibility matrix (daemon vs UI)
- [ ] API version negotiation (breaking changes)
