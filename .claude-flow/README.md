# Claude-Flow Integration for keyrx

## Overview

keyrx is now claude-flow ready! This integration provides specialized AI agents, domain definitions, and workflows tailored to the keyrx keyboard remapping system architecture.

## Quick Start

```bash
# Initialize memory database
npx claude-flow@v3alpha memory init

# Start background daemon (optional)
npx claude-flow@v3alpha daemon start

# Run a workflow
npx claude-flow@v3alpha flow run spec-implementation --spec uat-ui-fixes

# List available agents
npx claude-flow@v3alpha agent list

# Check swarm status
npx claude-flow@v3alpha status --watch
```

## Architecture

### Domains

keyrx is organized into 8 DDD domains (see `domains/keyrx-domains.json`):

1. **Core** - Platform-agnostic remapping logic (no_std)
2. **Compiler** - Rhai → .krx binary compilation
3. **Daemon** - OS integration + web server
4. **UI** - React + WASM frontend
5. **Platform** - OS-specific implementations (Linux/Windows)
6. **Testing** - DST, PBT, Fuzz, E2E testing
7. **Configuration** - Multi-device config management
8. **Quality** - Code quality enforcement

### Specialized Agents

20 specialized agents for keyrx development (see `agents/keyrx-agents.json`):

**Development Agents:**
- `rust-core-dev` - no_std core development
- `compiler-dev` - Rhai compiler implementation
- `daemon-dev` - Daemon + web server
- `platform-linux-dev` - evdev/uinput
- `platform-windows-dev` - Windows hooks + Raw Input
- `ui-react-dev` - React + TypeScript
- `wasm-dev` - WASM compilation + browser simulation

**Testing Agents:**
- `unit-tester` - Unit tests (cargo test, vitest)
- `dst-tester` - Deterministic Simulation Testing
- `pbt-tester` - Property-Based Testing (proptest)
- `fuzz-tester` - Fuzz testing (cargo-fuzz)
- `e2e-tester` - E2E tests (Playwright)
- `windows-vm-tester` - Vagrant VM testing

**Quality Agents:**
- `clippy-enforcer` - Linting (zero warnings)
- `formatter` - Code formatting (rustfmt, prettier)
- `coverage-checker` - Coverage analysis

**Spec Workflow Agents:**
- `spec-planner` - Parse specs, create tasks
- `spec-implementer` - Execute tasks, log artifacts
- `spec-reviewer` - Verify completion, request approvals

**Coordination:**
- `architecture-coordinator` - Multi-domain coordination
- `keyrx-strategic-queen` - Strategic planning (queen agent)

## Workflows

7 pre-configured workflows (see `flows/keyrx-workflows.json`):

### 1. spec-implementation

Implement a spec from `.spec-workflow`:

```bash
npx claude-flow@v3alpha flow run spec-implementation --spec uat-ui-fixes
```

**Stages:**
1. **Planning** - Load spec, analyze requirements, parse tasks
2. **Implementation** - Execute tasks, create artifacts
3. **Testing** - Unit tests, E2E tests, coverage check
4. **Quality Gates** - Clippy, format, coverage verification
5. **Review** - Verify artifacts, request approval

**Example for uat-ui-fixes spec (14 tasks):**
- Virtual/physical device indicator
- Enable/disable toggle (replace delete)
- Profile inline edit + active indicator
- Config page navigation sync
- RPC error fix
- 256-layer display
- Key dropdown population

### 2. feature-development

Full feature development cycle:

```bash
npx claude-flow@v3alpha flow run feature-development --feature "multi-device-support"
```

**Stages:**
1. **Planning** - Requirements gathering, architecture design
2. **Implementation** - Parallel development across domains
3. **Testing** - Comprehensive test suite (unit, DST, PBT, E2E)
4. **Quality** - All quality gates

### 3. bug-fix

Quick bug fix workflow:

```bash
npx claude-flow@v3alpha flow run bug-fix --issue 42
```

**Stages:**
1. **Investigation** - Debug and identify root cause
2. **Fix** - Implement the fix
3. **Verification** - Regression test + quality check

### 4. quality-verification

Full quality verification (equivalent to `make verify`):

```bash
npx claude-flow@v3alpha flow run quality-verification
```

**Stages:**
1. **Rust Quality** - Clippy + rustfmt
2. **TypeScript Quality** - Prettier
3. **Tests** - Backend + frontend + E2E
4. **Coverage** - Backend + frontend coverage checks

### 5. windows-vm-test

Windows platform testing via Vagrant:

```bash
npx claude-flow@v3alpha flow run windows-vm-test
```

**Stages:**
1. **VM Setup** - Start Vagrant VM
2. **Testing** - Run Windows-specific tests
3. **Cleanup** - Restore snapshot

### 6. wasm-build-verify

WASM build and verification:

```bash
npx claude-flow@v3alpha flow run wasm-build-verify
```

**Stages:**
1. **Build** - Build WASM module
2. **Verification** - Verify integrity + health check

## Integration with .spec-workflow

Claude-flow is fully integrated with your existing `.spec-workflow` system (see `integration/spec-workflow-integration.json`):

### Automatic Discovery

- Scans `.spec-workflow/specs/` for specs
- Parses `requirements.md`, `design.md`, `tasks.md`
- Tracks active specs and progress

### Task Execution

Agents understand spec task format:

```markdown
- [x] 1.1 Add device type detection to API/types
  - File: `src/types/index.ts`, `src/api/devices.ts`
  - Purpose: Distinguish daemon-created virtual keyboards from physical hardware
  - _Leverage: src/types/index.ts, src/api/devices.ts_
  - _Requirements: Dashboard shows clear indicator for virtual vs physical device_
  - _Prompt: Implement the task for spec uat-ui-fixes..._
```

Agents will:
1. Mark task as `[-]` (in-progress)
2. Read files specified in task
3. Implement changes
4. Log artifacts using `mcp__spec-workflow__log-implementation`
5. Mark task as `[x]` (completed)

### Implementation Logging

Required artifact types:
- **apiEndpoints** - REST API endpoints
- **components** - React components
- **functions** - Utility functions
- **classes** - Class definitions
- **integrations** - Frontend-backend integrations

Example:

```json
{
  "specName": "uat-ui-fixes",
  "taskId": "1.1",
  "summary": "Added device type detection with isVirtual field",
  "filesModified": ["src/types/index.ts", "src/api/devices.ts"],
  "filesCreated": [],
  "statistics": {
    "linesAdded": 25,
    "linesRemoved": 3
  },
  "artifacts": {
    "apiEndpoints": [],
    "components": [],
    "functions": [
      {
        "name": "detectVirtualDevice",
        "purpose": "Detect if device is virtual based on name",
        "location": "src/api/devices.ts:42",
        "signature": "(deviceName: string) => boolean",
        "isExported": true
      }
    ]
  }
}
```

### Approval Workflow

Agents can request approvals:

```bash
# Agent requests approval
mcp__spec-workflow__approvals --action request \
  --category spec \
  --categoryName uat-ui-fixes \
  --type document \
  --title "Task 1.1 Complete" \
  --filePath ".spec-workflow/specs/uat-ui-fixes/tasks.md"

# Check approval status
mcp__spec-workflow__approvals --action status --approvalId <id>

# Delete after approval
mcp__spec-workflow__approvals --action delete --approvalId <id>
```

## Memory & Learning

Claude-flow learns from your codebase:

### Pattern Storage

Agents store successful patterns:
- API endpoint creation patterns
- React component structure
- Rust module organization
- Test writing patterns

### Vector Search

HNSW-based vector search for:
- Finding similar implementations
- Discovering existing APIs
- Avoiding code duplication
- Reusing proven patterns

### Knowledge Base

Agents build a knowledge base from:
- Implementation logs
- Code structure
- Test patterns
- Error resolutions

## Quality Gates

All workflows enforce keyrx quality standards:

### Code Limits
- **500 lines max per file** (excluding comments/blanks)
- **50 lines max per function**
- **80% test coverage minimum** (90% for keyrx_core)

### Quality Checks
- **Clippy** - Zero warnings (`-D warnings`)
- **Format** - rustfmt + prettier
- **Tests** - 100% pass rate (backend), 95% pass rate (frontend)
- **Coverage** - 80% minimum, 90% for core
- **Accessibility** - Zero WCAG violations

### Enforcement
- Pre-commit hooks
- CI/CD gates
- Automated verification

## Swarm Coordination

### Hierarchical-Mesh Topology

Default topology combines:
- **Strategic Queen** - High-level planning and coordination
- **Worker Agents** - Specialized domain experts
- **Consensus** - Byzantine fault tolerance (67% threshold)

### Auto-Scaling

Swarm automatically scales workers based on:
- Task complexity
- Domain requirements
- Available resources

Max 15 concurrent agents by default (configurable in `config.yaml`).

## Commands

### Agent Management

```bash
# List all agents
npx claude-flow@v3alpha agent list

# Spawn specific agent
npx claude-flow@v3alpha agent spawn -t rust-core-dev --name feature-dev

# Get agent pool status
npx claude-flow@v3alpha agent pool --health
```

### Flow Execution

```bash
# Run a flow
npx claude-flow@v3alpha flow run <flow-name> [--args]

# List available flows
npx claude-flow@v3alpha flow list

# Get flow status
npx claude-flow@v3alpha flow status <flow-id>
```

### Swarm Management

```bash
# Initialize swarm
npx claude-flow@v3alpha swarm init --v3-mode

# Get swarm status
npx claude-flow@v3alpha status --watch

# Scale swarm
npx claude-flow@v3alpha swarm scale --max-agents 20
```

### Memory Management

```bash
# Initialize memory
npx claude-flow@v3alpha memory init

# Search patterns
npx claude-flow@v3alpha memory search -q "API endpoint" --top-k 5

# Store pattern
npx claude-flow@v3alpha memory store --pattern "react-component" --success-rate 0.99

# Consolidate memory
npx claude-flow@v3alpha memory consolidate --threshold 0.7
```

### Health & Diagnostics

```bash
# Run health checks
npx claude-flow@v3alpha doctor --fix

# View metrics
npx claude-flow@v3alpha performance benchmark --suite all

# Check security
npx claude-flow@v3alpha security scan --depth full
```

## Configuration

### Global Config

Edit `.claude-flow/config.yaml`:

```yaml
version: "3.0.0"

swarm:
  topology: hierarchical-mesh
  maxAgents: 15
  autoScale: true
  coordinationStrategy: consensus

memory:
  backend: hybrid
  enableHNSW: true
  persistPath: .claude-flow/data
  cacheSize: 100

neural:
  enabled: true
  modelPath: .claude-flow/neural

hooks:
  enabled: true
  autoExecute: true
```

### Domain-Specific Config

Each domain can have custom config in `domains/<domain>.json`.

### Agent Config

Customize agent behavior in `agents/keyrx-agents.json`:

```json
{
  "name": "rust-core-dev",
  "model": "opus",
  "temperature": 0.7,
  "maxTokens": 8000,
  "constraints": {
    "noStd": true,
    "latencyBudget": "100μs",
    "coverage": "90%"
  }
}
```

## Best Practices

### 1. Use Appropriate Agents

- **Simple edits** → `rust-core-dev`, `ui-react-dev`
- **Architecture changes** → `architecture-coordinator`
- **Testing** → `unit-tester`, `dst-tester`, `pbt-tester`
- **Spec execution** → `spec-implementer`

### 2. Follow Workflows

Use pre-defined workflows instead of manual agent spawning:

```bash
# Good
npx claude-flow@v3alpha flow run spec-implementation --spec uat-ui-fixes

# Manual (not recommended)
npx claude-flow@v3alpha agent spawn -t spec-planner
npx claude-flow@v3alpha agent spawn -t spec-implementer
# ... etc
```

### 3. Log Artifacts

Always use `mcp__spec-workflow__log-implementation` to create searchable knowledge base.

### 4. Request Approvals

Use approval workflow for significant changes:

```bash
mcp__spec-workflow__approvals --action request \
  --category spec \
  --type document \
  --title "Feature XYZ Complete"
```

### 5. Monitor Progress

```bash
# Watch swarm activity
npx claude-flow@v3alpha status --watch

# Check spec progress
mcp__spec-workflow__spec-status --specName uat-ui-fixes
```

## Troubleshooting

### Workflow Failures

```bash
# Check logs
cat .claude-flow/logs/latest.log

# Check agent status
npx claude-flow@v3alpha agent pool --health

# Restart swarm
npx claude-flow@v3alpha swarm restart
```

### Memory Issues

```bash
# Clear cache
npx claude-flow@v3alpha memory clear --cache-only

# Rebuild index
npx claude-flow@v3alpha memory rebuild
```

### Agent Errors

```bash
# Run diagnostics
npx claude-flow@v3alpha doctor --fix

# Check agent config
cat .claude-flow/agents/keyrx-agents.json
```

## Integration with Existing Workflow

Claude-flow complements your existing development workflow:

### Make Commands

```bash
make setup      # Install tools + initialize claude-flow
make build      # Build workspace
make test       # Run tests
make verify     # Full quality checks (uses quality-verification flow)
```

### Git Hooks

Pre-commit hooks automatically run `quality-verification` flow.

### CI/CD

GitHub Actions can trigger claude-flow workflows:

```yaml
- name: Run quality verification
  run: npx claude-flow@v3alpha flow run quality-verification
```

## Examples

### Implement Active Spec

```bash
# Current active spec: uat-ui-fixes (14 tasks)
npx claude-flow@v3alpha flow run spec-implementation --spec uat-ui-fixes

# Agent will:
# 1. Parse tasks.md (14 tasks)
# 2. Execute each task sequentially
# 3. Log implementation artifacts
# 4. Run tests
# 5. Verify quality gates
# 6. Request approval
```

### Add New Feature

```bash
# Start feature development
npx claude-flow@v3alpha flow run feature-development --feature "macro-recording"

# Agent will:
# 1. Create spec in .spec-workflow/specs/macro-recording/
# 2. Write requirements.md, design.md, tasks.md
# 3. Request approval
# 4. After approval: implement tasks
# 5. Run comprehensive tests
# 6. Verify quality gates
```

### Fix Bug

```bash
# Quick bug fix
npx claude-flow@v3alpha flow run bug-fix --issue "RPC error on config save"

# Agent will:
# 1. Debug and identify root cause
# 2. Implement fix
# 3. Create regression test
# 4. Run quality checks
```

### Windows Testing

```bash
# Test on Windows VM
npx claude-flow@v3alpha flow run windows-vm-test

# Agent will:
# 1. Start Vagrant VM
# 2. Run Windows-specific tests
# 3. Restore snapshot
```

## Resources

- **Claude-Flow Docs** - https://github.com/ruvnet/claude-flow
- **keyrx Steering Docs** - `.spec-workflow/steering/`
- **Active Specs** - `.spec-workflow/specs/`
- **Agent Definitions** - `.claude-flow/agents/keyrx-agents.json`
- **Domain Definitions** - `.claude-flow/domains/keyrx-domains.json`
- **Workflow Definitions** - `.claude-flow/flows/keyrx-workflows.json`

## Support

For issues with:
- **Claude-flow** - https://github.com/ruvnet/claude-flow/issues
- **keyrx** - https://github.com/rmondo/keyrx/issues
- **Integration** - Check `.claude-flow/logs/` and run `npx claude-flow@v3alpha doctor`
