# Troubleshooting Guide: E2E Test Suite

Comprehensive troubleshooting guide for common issues when running the automated E2E test suite.

## Table of Contents

1. [Dependency Issues](#dependency-issues)
2. [Daemon Issues](#daemon-issues)
3. [Test Failures](#test-failures)
4. [WebSocket Issues](#websocket-issues)
5. [Performance Issues](#performance-issues)
6. [CI/CD Issues](#cicd-issues)
7. [Environment Issues](#environment-issues)

---

## Dependency Issues

### Error: Cannot find module 'zod'

**Symptom:**
```
Error [ERR_MODULE_NOT_FOUND]: Cannot find package 'zod'
```

**Cause:** Missing dependencies in `package.json`

**Solution:**
```bash
# Install missing dependencies
npm install

# Or install specific dependency
npm install zod axios ws deep-diff commander

# Verify installation
npm list zod
```

**Prevention:**
- Always run `npm install` after pulling changes
- Check `package.json` includes all required dependencies

### Error: TypeScript compilation failed

**Symptom:**
```
TSError: Cannot find name 'WebSocket'
Property 'baseUrl' does not exist on type '{}'
```

**Cause:** Missing type definitions

**Solution:**
```bash
# Install type definitions
npm install --save-dev @types/node @types/ws

# Verify tsconfig.json includes correct settings
cat tsconfig.json  # Should include "types": ["node"]
```

### Error: Module resolution failed

**Symptom:**
```
Error: Cannot find module './client.js'
```

**Cause:** Incorrect import extensions or paths

**Solution:**
- Use `.js` extensions for TypeScript imports: `import { X } from './file.js'`
- Check file exists at the import path
- Verify `tsconfig.json` has `"moduleResolution": "node16"` or `"nodenext"`

---

## Daemon Issues

### Error: Daemon failed to start

**Symptom:**
```
❌ Error: Failed to start daemon: daemon process exited with code 1
```

**Cause:** Daemon binary missing, wrong path, or startup failure

**Solutions:**

1. **Verify daemon exists:**
   ```bash
   ls -lh target/release/keyrx_daemon
   # Should show executable file
   ```

2. **Rebuild daemon:**
   ```bash
   cargo build --release -p keyrx_daemon
   ```

3. **Check daemon runs manually:**
   ```bash
   ./target/release/keyrx_daemon
   # Should start without errors
   ```

4. **Check for port conflicts:**
   ```bash
   lsof -i :9867
   # Kill conflicting process:
   lsof -ti:9867 | xargs kill -9
   ```

5. **Use different port:**
   ```bash
   npm run test:e2e:auto -- --port 9868
   ```

### Error: Daemon health check timeout

**Symptom:**
```
❌ Daemon failed to become healthy within 30000ms
```

**Cause:** Daemon starting slowly or not responding to health checks

**Solutions:**

1. **Check daemon logs:**
   - Logs are printed in test output
   - Look for startup errors or warnings

2. **Test health endpoint manually:**
   ```bash
   # Start daemon manually
   ./target/release/keyrx_daemon &

   # Test health check
   curl http://localhost:9867/api/health

   # Should return: {"status":"ok","version":"..."}
   ```

3. **Increase timeout in fixture:**
   - Edit `scripts/fixtures/daemon-fixture.ts`
   - Increase `HEALTH_CHECK_TIMEOUT` value (default: 30000ms)

4. **Check system resources:**
   ```bash
   # CPU and memory usage
   top

   # Disk space
   df -h
   ```

### Error: Daemon process won't stop

**Symptom:**
- Daemon still running after tests
- Port remains in use
- Next test run fails with "Address already in use"

**Solutions:**

1. **Find and kill daemon process:**
   ```bash
   # Find process
   ps aux | grep keyrx_daemon

   # Kill by PID
   kill -9 <PID>

   # Or kill by port
   lsof -ti:9867 | xargs kill -9
   ```

2. **Kill all daemon instances:**
   ```bash
   pkill -9 keyrx_daemon
   ```

3. **Wait for cleanup:**
   - Daemon fixture has 5-second grace period
   - May need to wait before retrying tests

---

## Test Failures

### Error: Response schema mismatch

**Symptom:**
```
❌ profiles-003: POST /api/profiles - create profile
Expected: { "profile": { "name": "test-profile" } }
Actual:   { "success": true, "profile": { "name": "test-profile", "id": 123 } }
```

**Cause:** API response changed, expected results outdated

**Solutions:**

1. **Verify API change is intentional:**
   - Check recent commits
   - Review API documentation
   - Test endpoint manually

2. **Update expected results:**
   ```bash
   # Edit scripts/fixtures/expected-results.json
   # Update the expected response structure
   ```

3. **Use auto-fix to suggest update:**
   ```bash
   npm run test:e2e:auto -- --fix
   # Review suggested changes carefully
   ```

4. **Add ignore fields for dynamic values:**
   ```typescript
   assert: (response, expected) => {
     return comparator.compare(response, expected, {
       ignoreFields: ['id', 'timestamp', 'created_at']
     });
   }
   ```

### Error: Test cleanup failed

**Symptom:**
```
❌ Cleanup failed: Profile 'test-profile' not found
```

**Cause:** Test failed before cleanup, resource already deleted, or previous test leaked state

**Solutions:**

1. **Make cleanup idempotent:**
   ```typescript
   cleanup: async () => {
     try {
       await client.deleteProfile('test-profile');
     } catch (error) {
       // Already deleted or doesn't exist - that's fine
       if (error.status !== 404) {
         throw error;  // Re-throw unexpected errors
       }
     }
   }
   ```

2. **Add setup to ensure clean state:**
   ```typescript
   setup: async () => {
     // Delete if exists
     try {
       await client.deleteProfile('test-profile');
     } catch {
       // Doesn't exist, good
     }
   }
   ```

3. **Use unique test identifiers:**
   ```typescript
   const testId = `test-${Date.now()}`;
   await client.createProfile(testId);
   ```

### Error: Assertion failed with no diff

**Symptom:**
```
❌ Test failed but no difference shown
Expected: [complex object]
Actual:   [complex object]
```

**Cause:** Deep comparison issue or floating-point mismatch

**Solutions:**

1. **Enable verbose logging:**
   ```typescript
   // In test executor
   const result = comparator.compare(response, expected, {
     verbose: true,
     showFullDiff: true
   });
   ```

2. **Check for floating-point issues:**
   ```typescript
   // Don't compare exact floats
   assert: (response, expected) => {
     const diff = Math.abs(response.value - expected.value);
     return diff < 0.001;  // Tolerance
   }
   ```

3. **Inspect actual values:**
   ```typescript
   execute: async (client) => {
     const result = await client.getStatus();
     console.log('Actual response:', JSON.stringify(result, null, 2));
     return result;
   }
   ```

---

## WebSocket Issues

### Error: WebSocket connection failed

**Symptom:**
```
❌ websocket-001: Failed to connect to WebSocket
Error: Connection refused
```

**Cause:** Daemon not running, wrong URL, or WebSocket not enabled

**Solutions:**

1. **Verify daemon is running:**
   ```bash
   curl http://localhost:9867/api/health
   ```

2. **Check WebSocket endpoint:**
   ```bash
   # Install wscat
   npm install -g wscat

   # Test WebSocket connection
   wscat -c ws://localhost:9867/ws
   # Should connect successfully
   ```

3. **Verify WebSocket URL:**
   - Should be `ws://` not `http://`
   - Port should match daemon port (default: 9867)
   - Path should be `/ws`

4. **Check firewall:**
   ```bash
   # Linux
   sudo ufw status

   # macOS
   sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate
   ```

### Error: WebSocket events not received

**Symptom:**
```
❌ Timeout waiting for WebSocket event
No event received within 5000ms
```

**Cause:** Not subscribed to channel, event not triggered, or daemon not broadcasting

**Solutions:**

1. **Verify subscription:**
   ```typescript
   // Subscribe before triggering event
   await wsClient.connect();
   await wsClient.subscribe('devices');  // Must subscribe first!
   await client.updateDevice('id', { enabled: false });  // Then trigger
   const event = await wsClient.waitForEvent(...);
   ```

2. **Check channel name:**
   - Valid channels: `devices`, `profiles`, `metrics`, `state`
   - Channel names are case-sensitive

3. **Increase timeout:**
   ```typescript
   const event = await wsClient.waitForEvent(
     predicate,
     10000  // Increase from 5000ms to 10000ms
   );
   ```

4. **Debug event messages:**
   ```typescript
   wsClient.on('message', (msg) => {
     console.log('Received:', JSON.stringify(msg, null, 2));
   });
   ```

### Error: WebSocket reconnection failed

**Symptom:**
```
❌ Reconnection test failed: subscriptions not restored
```

**Cause:** Reconnection logic not implemented or subscriptions not persisted

**Solutions:**

1. **Verify reconnection support:**
   - Check if daemon supports reconnection
   - May need to manually resubscribe

2. **Manual resubscription:**
   ```typescript
   await wsClient.disconnect();
   await wsClient.connect();
   // Resubscribe manually
   await wsClient.subscribe('devices');
   ```

3. **Use automatic reconnection:**
   ```typescript
   const wsClient = new WebSocketClient(url, {
     autoReconnect: true,
     reconnectInterval: 1000,
     restoreSubscriptions: true
   });
   ```

---

## Performance Issues

### Issue: Tests running too slowly

**Symptom:**
- Test suite takes > 3 minutes
- Individual tests taking > 5 seconds

**Solutions:**

1. **Profile slow tests:**
   ```bash
   npm run test:e2e:auto
   # Check "Slowest tests" section in output
   ```

2. **Reduce unnecessary waits:**
   ```typescript
   // Bad: Fixed sleep
   await sleep(5000);

   // Good: Poll with short interval
   for (let i = 0; i < 50; i++) {
     const status = await client.getStatus();
     if (status.ready) break;
     await sleep(100);
   }
   ```

3. **Reuse API clients:**
   ```typescript
   // Bad: New client per call
   async function cleanup() {
     const client = new ApiClient(config);
     await client.deleteProfile('test');
   }

   // Good: Reuse client
   const client = new ApiClient(config);
   async function cleanup() {
     await client.deleteProfile('test');
   }
   ```

4. **Optimize daemon startup:**
   - Use already-running daemon for development
   - Cache daemon binary in CI

### Issue: Tests timing out

**Symptom:**
```
❌ Test exceeded timeout of 30000ms
```

**Solutions:**

1. **Increase test timeout:**
   ```typescript
   const executor = new TestExecutor({
     testTimeout: 60000,  // Increase to 60 seconds
   });
   ```

2. **Increase operation timeout:**
   ```typescript
   const client = new ApiClient({
     baseUrl,
     timeout: 10000,  // 10 second timeout for requests
   });
   ```

3. **Check for infinite loops:**
   - Review test logic for potential hangs
   - Add timeout to polling loops

---

## CI/CD Issues

### Error: Tests pass locally but fail in CI

**Symptom:**
- All tests pass on local machine
- Same tests fail in GitHub Actions

**Causes & Solutions:**

1. **Resource constraints in CI:**
   - CI runners may be slower
   - Increase timeouts for CI environment
   - Use conditional timeouts:
     ```typescript
     const timeout = process.env.CI ? 60000 : 30000;
     ```

2. **Missing dependencies:**
   ```yaml
   # .github/workflows/e2e-auto.yml
   - name: Install dependencies
     run: npm install
     # Ensure all deps installed before tests
   ```

3. **Environment differences:**
   - Check environment variables
   - Verify file paths (absolute vs relative)
   - Check OS differences (Linux in CI vs macOS local)

4. **Timing-sensitive tests:**
   - Add retry logic for flaky tests
   - Use longer timeouts in CI
   - Make tests more deterministic

### Error: Artifacts not uploaded

**Symptom:**
- Test results not available in GitHub
- No JSON report in artifacts

**Solutions:**

1. **Check workflow syntax:**
   ```yaml
   - name: Upload results
     uses: actions/upload-artifact@v3
     if: always()  # Upload even if tests fail
     with:
       name: test-results
       path: test-results.json
   ```

2. **Verify file exists:**
   ```yaml
   - name: Check results file
     run: ls -lh test-results.json
   ```

3. **Use correct paths:**
   - Use relative paths from repository root
   - Check file was generated in correct location

---

## Environment Issues

### Issue: Different results on different machines

**Symptom:**
- Tests pass on machine A
- Same tests fail on machine B
- Results inconsistent

**Causes & Solutions:**

1. **Different Node.js versions:**
   ```bash
   node --version
   # Ensure same version (18+ required)
   # Use nvm to switch: nvm use 18
   ```

2. **Different Rust versions:**
   ```bash
   rustc --version
   # Update: rustup update
   ```

3. **Different OS behavior:**
   - File path separators (/ vs \)
   - Line endings (LF vs CRLF)
   - Case sensitivity (macOS is case-insensitive by default)

4. **System-specific issues:**
   ```bash
   # Linux: check available resources
   free -h
   df -h

   # macOS: check open file limits
   ulimit -n
   ```

### Issue: Permission denied errors

**Symptom:**
```
Error: EACCES: permission denied, open 'metrics.jsonl'
```

**Solutions:**

1. **Fix file permissions:**
   ```bash
   chmod 644 metrics.jsonl
   chmod +x target/release/keyrx_daemon
   ```

2. **Check directory permissions:**
   ```bash
   ls -la scripts/
   chmod 755 scripts/
   ```

3. **Run without sudo:**
   - Don't use `sudo npm run test:e2e:auto`
   - Files created by sudo need permission changes

---

## Getting More Help

If your issue isn't covered here:

1. **Check logs:**
   - Daemon output in test results
   - Browser console (for WebSocket issues)
   - System logs (`journalctl -xe` on Linux)

2. **Enable verbose mode:**
   ```bash
   DEBUG=* npm run test:e2e:auto
   ```

3. **Test manually:**
   - Run daemon manually
   - Test endpoints with `curl` or Postman
   - Isolate the failing component

4. **Review recent changes:**
   ```bash
   git log --oneline -10
   git diff HEAD~1
   ```

5. **Check documentation:**
   - [README.md](./README.md) - System overview
   - [DEV_GUIDE.md](./DEV_GUIDE.md) - Development guide
   - Project root `CLAUDE.md` - Development guidelines

6. **File an issue:**
   - Include full error message
   - Include test output
   - Include system info (OS, Node version, Rust version)
   - Include steps to reproduce
