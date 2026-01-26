# Claude-Flow Quick Start for keyrx

## 5-Minute Setup

```bash
# 1. Initialize memory database
npx claude-flow@v3alpha memory init

# 2. Load keyrx patterns
npx claude-flow@v3alpha memory load .claude-flow/memory/keyrx-patterns.json

# 3. Verify setup
npx claude-flow@v3alpha doctor

# 4. Run your first workflow
npx claude-flow@v3alpha flow run quality-verification
```

## Common Commands

### Implement Active Spec

```bash
# See active specs
cat .claude/.claude/CLAUDE.md | grep "Active Specs" -A 5

# Run spec implementation (example: uat-ui-fixes)
npx claude-flow@v3alpha flow run spec-implementation --spec uat-ui-fixes
```

### Quality Check Before Commit

```bash
# Quick quality verification
npx claude-flow@v3alpha flow run quality-verification

# Or use make command
make verify
```

### Bug Fix

```bash
# Quick bug fix workflow
npx claude-flow@v3alpha flow run bug-fix --issue "description"
```

### Windows Testing

```bash
# Test on Windows VM
npx claude-flow@v3alpha flow run windows-vm-test
```

### WASM Build

```bash
# Build and verify WASM
npx claude-flow@v3alpha flow run wasm-build-verify
```

## Agent Quick Reference

### Development
- `rust-core-dev` - Core library (no_std)
- `daemon-dev` - Daemon + web server
- `ui-react-dev` - React frontend
- `compiler-dev` - Rhai compiler

### Testing
- `unit-tester` - Unit tests
- `e2e-tester` - E2E tests
- `dst-tester` - Deterministic simulation
- `fuzz-tester` - Fuzz testing

### Quality
- `clippy-enforcer` - Linting
- `formatter` - Code formatting
- `coverage-checker` - Coverage analysis

### Spec Workflow
- `spec-planner` - Parse specs
- `spec-implementer` - Execute tasks
- `spec-reviewer` - Verify & approve

## Workflow Quick Reference

| Workflow | Command | Use Case |
|----------|---------|----------|
| `spec-implementation` | `flow run spec-implementation --spec <name>` | Implement .spec-workflow specs |
| `feature-development` | `flow run feature-development --feature <name>` | Full feature cycle |
| `bug-fix` | `flow run bug-fix --issue <description>` | Quick bug fixes |
| `quality-verification` | `flow run quality-verification` | Pre-commit checks |
| `windows-vm-test` | `flow run windows-vm-test` | Windows platform testing |
| `wasm-build-verify` | `flow run wasm-build-verify` | WASM build & verify |

## Monitoring

```bash
# Watch swarm activity
npx claude-flow@v3alpha status --watch

# Check memory patterns
npx claude-flow@v3alpha memory search -q "API endpoint"

# View logs
tail -f .claude-flow/logs/latest.log
```

## Help

```bash
# General help
npx claude-flow@v3alpha --help

# Command-specific help
npx claude-flow@v3alpha flow --help
npx claude-flow@v3alpha agent --help
npx claude-flow@v3alpha memory --help
```

## Next Steps

1. Read full documentation: `.claude-flow/README.md`
2. Review domain definitions: `.claude-flow/domains/keyrx-domains.json`
3. Explore agent capabilities: `.claude-flow/agents/keyrx-agents.json`
4. Customize workflows: `.claude-flow/flows/keyrx-workflows.json`
5. Check spec integration: `.claude-flow/integration/spec-workflow-integration.json`

## Examples

### Example 1: Implement Current Spec

```bash
# Current spec: uat-ui-fixes (14 tasks)
npx claude-flow@v3alpha flow run spec-implementation --spec uat-ui-fixes

# Monitor progress
npx claude-flow@v3alpha status --watch

# Check spec status
npx mcp-client mcp__spec-workflow__spec-status --specName uat-ui-fixes
```

### Example 2: Add New API Endpoint

```bash
# Spawn daemon development agent
npx claude-flow@v3alpha agent spawn -t daemon-dev --name api-dev

# Agent will:
# - Follow api-endpoint-creation pattern
# - Create handler function
# - Add route to Router
# - Add error handling
# - Write tests
# - Log implementation artifacts
```

### Example 3: Fix Failing Tests

```bash
# Debug with unit-tester agent
npx claude-flow@v3alpha agent spawn -t unit-tester --name test-fixer

# Agent will:
# - Analyze test failures
# - Identify root cause
# - Fix issues
# - Verify tests pass
# - Check coverage
```

### Example 4: Full Feature Development

```bash
# Start new feature
npx claude-flow@v3alpha flow run feature-development --feature "macro-recording"

# Workflow stages:
# 1. Planning - Create spec with requirements, design, tasks
# 2. Implementation - Parallel development across domains
# 3. Testing - Comprehensive test suite
# 4. Quality - All quality gates
# 5. Review - Verification and approval
```

## Troubleshooting

### Workflow fails

```bash
# Check logs
cat .claude-flow/logs/latest.log

# Run diagnostics
npx claude-flow@v3alpha doctor --fix
```

### Agent errors

```bash
# Check agent status
npx claude-flow@v3alpha agent pool --health

# Restart swarm
npx claude-flow@v3alpha swarm restart
```

### Memory issues

```bash
# Clear cache
npx claude-flow@v3alpha memory clear --cache-only

# Rebuild patterns
npx claude-flow@v3alpha memory rebuild
```

## Tips

1. **Use workflows over manual agents** - Workflows handle coordination automatically
2. **Monitor with `--watch`** - See real-time progress
3. **Check logs for errors** - `.claude-flow/logs/latest.log`
4. **Search memory patterns** - Learn from past implementations
5. **Request approvals** - Use approval workflow for significant changes

## Support

- **Full docs** - `.claude-flow/README.md`
- **Claude-flow** - https://github.com/ruvnet/claude-flow
- **keyrx** - `.spec-workflow/steering/`
