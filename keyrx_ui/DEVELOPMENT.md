# KeyRx UI Development Guide

## Quick Start (Recommended)

```bash
# Start development environment with HMR
./scripts/dev_ui.sh
```

This automatically:
- Starts the daemon (if not running)
- Launches Vite dev server with HMR on port 5173
- Proxies API/WebSocket requests to daemon
- Opens browser automatically

**React changes reload instantly!** No rebuild needed.

## Development Modes

### 1. Development Mode (Fast Iteration)

**Use this 99% of the time for UI development.**

```bash
# Terminal 1: Start daemon (once)
./scripts/UAT.sh

# Terminal 2: Start Vite dev server with HMR
cd keyrx_ui
npm run dev
```

Or use the all-in-one script:
```bash
./scripts/dev_ui.sh
```

**What you get:**
- ✅ Instant Hot Module Replacement (HMR)
- ✅ React changes reload in <100ms
- ✅ Full TypeScript type checking
- ✅ React DevTools support
- ✅ API/WebSocket proxy to daemon (no CORS issues)

**URLs:**
- UI Dev Server: http://localhost:5173 (use this for development)
- Daemon API: http://localhost:9867 (background service)

### 2. Production Build (Testing Final Bundle)

**Use this only when:**
- Testing production performance
- Verifying bundle size
- Testing with embedded UI in daemon

```bash
# Build UI
./scripts/build_ui.sh

# Rebuild daemon to embed UI
cargo build -p keyrx_daemon --features linux

# Install and restart daemon
cp target/debug/keyrx_daemon ~/.local/bin/keyrx_daemon
pkill keyrx_daemon
~/.local/bin/keyrx_daemon run --config ~/.config/keyrx/config.krx
```

## Development Workflow

### For React/TypeScript Changes

**No rebuild needed!** Just save the file and see changes instantly.

1. Edit React/TypeScript files in `keyrx_ui/src/`
2. Save
3. Browser auto-reloads with changes (< 100ms)

**Example:**
```typescript
// Edit keyrx_ui/src/pages/ConfigPage.tsx
// Save file
// Browser updates instantly!
```

### For WASM Changes

If you modify `keyrx_core` Rust code that affects WASM:

```bash
cd keyrx_ui
npm run build:wasm
```

The Vite dev server will detect the new WASM and reload automatically.

### For Daemon (Rust) Changes

If you modify daemon Rust code:

```bash
# Rebuild daemon
cargo build -p keyrx_daemon --features linux

# Restart daemon (Terminal 1 or background)
pkill keyrx_daemon
./target/debug/keyrx_daemon run --config ~/.config/keyrx/config.krx
```

The Vite dev server continues running - just refresh the browser.

## Proxy Configuration

Vite dev server proxies these requests to the daemon:

- **API requests:** `/api/*` → `http://localhost:9867/api/*`
- **WebSocket:** `/ws` → `ws://localhost:9867/ws`

This is configured in `vite.config.ts`:

```typescript
server: {
  port: 5173,
  proxy: {
    '/api': {
      target: 'http://localhost:9867',
      changeOrigin: true,
    },
    '/ws': {
      target: 'ws://localhost:9867',
      ws: true,
      changeOrigin: true,
    },
  },
}
```

## Troubleshooting

### "Daemon not responding on port 9867"

**Problem:** Daemon is not running or crashed.

**Solution:**
```bash
# Check if daemon is running
pgrep keyrx_daemon

# Check daemon logs
tail -f /tmp/keyrx_daemon_dev.log

# Restart daemon
./scripts/UAT.sh
```

### "API requests fail with 404"

**Problem:** Proxy not configured or daemon API endpoint missing.

**Solution:**
1. Verify daemon is running: `curl http://localhost:9867/api/profiles`
2. Check Vite dev server logs for proxy errors
3. Ensure you're using `http://localhost:5173` (not 9867)

### "WebSocket connection fails"

**Problem:** WebSocket proxy not working.

**Solution:**
1. Check daemon WebSocket endpoint: `ws://localhost:9867/ws`
2. Verify proxy configuration in `vite.config.ts`
3. Check browser console for WebSocket errors

### "WASM module not found"

**Problem:** WASM hasn't been built yet.

**Solution:**
```bash
cd keyrx_ui
npm run build:wasm
```

### "Changes don't reload"

**Problem:** HMR not working.

**Solution:**
1. Check Vite dev server logs for errors
2. Hard refresh: `Ctrl+Shift+R` (or `Cmd+Shift+R` on Mac)
3. Restart Vite dev server: `Ctrl+C` then `npm run dev`

## Performance Comparison

| Mode | React Change Feedback | Full Rebuild Time |
|------|----------------------|-------------------|
| **Dev Mode (HMR)** | < 100ms ⚡ | N/A (no rebuild) |
| **Production Build** | ~30 seconds 🐌 | ~30 seconds |

**Development mode is 300x faster!**

## Best Practices

### Do ✅

- **Use dev mode** for all UI development
- **Run daemon once** and leave it running
- **Let HMR handle reloads** - don't manually refresh
- **Check browser console** for errors
- **Use React DevTools** for debugging

### Don't ❌

- **Don't rebuild UI** for every React change
- **Don't rebuild daemon** for UI changes
- **Don't use production mode** for development
- **Don't edit files in `dist/`** (auto-generated)

## Scripts Reference

| Script | Purpose | When to Use |
|--------|---------|-------------|
| `./scripts/dev_ui.sh` | All-in-one dev environment | Start of dev session |
| `npm run dev` | Vite dev server only | When daemon is already running |
| `npm run build:wasm` | Build WASM module | After keyrx_core changes |
| `./scripts/build_ui.sh` | Production build | Before testing final bundle |
| `./scripts/UAT.sh` | Start daemon for testing | Testing daemon functionality |

## Advanced: Custom Dev Server Port

To use a different port for Vite dev server:

```bash
cd keyrx_ui
npm run dev -- --port 3000
```

Update API calls if needed (or use environment variables).

## Environment Variables

Create `.env.local` in `keyrx_ui/` for custom configuration:

```bash
# Daemon API URL (if not using proxy)
VITE_API_URL=http://localhost:9867

# WebSocket URL
VITE_WS_URL=ws://localhost:9867/ws
```

## See Also

- [Vite Documentation](https://vitejs.dev/)
- [React Hot Module Replacement](https://vitejs.dev/guide/features.html#hot-module-replacement)
- [Proxy Configuration](https://vitejs.dev/config/server-options.html#server-proxy)
