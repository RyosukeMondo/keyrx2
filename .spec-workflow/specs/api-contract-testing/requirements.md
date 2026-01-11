# Requirements: API Contract Testing

## Overview

Establish automated API contract validation to ensure frontend Zod schemas match backend Rust API responses. Prevent schema mismatches from reaching production by catching them in CI/CD.

## Problem Statement

Schema mismatches between frontend and backend cause runtime errors during UAT:
- Frontend expects `source` but backend returns `config`
- Frontend expects required fields that backend returns as `null`
- Changes to backend API structs not reflected in frontend schemas

Current state: No automated detection of contract mismatches until manual UAT.

## User Stories

### US-1: Developer runs contract validation locally
**As a** developer
**I want to** run a command to validate API contracts against the live daemon
**So that** I can catch schema mismatches before committing code

**Acceptance Criteria:**
- EARS: When `npm run validate:contracts` is executed with daemon running, the system SHALL test all documented endpoints and report pass/fail for each
- Validation covers: GET /api/devices, GET /api/profiles, GET /api/profiles/:name/config
- Clear error messages identify which field mismatched

### US-2: CI automatically validates contracts
**As a** maintainer
**I want** CI to automatically run contract validation on every PR
**So that** mismatches are caught before merge

**Acceptance Criteria:**
- EARS: When a PR is created or updated, the CI pipeline SHALL run contract validation tests
- CI fails if any contract validation fails
- CI output shows which endpoints/fields failed

### US-3: New endpoints require contract tests
**As a** developer
**I want** a pattern for adding contract tests for new endpoints
**So that** all endpoints are covered automatically

**Acceptance Criteria:**
- EARS: When a new endpoint is added, there SHALL be a documented process to add contract validation
- Contract validation script is easily extensible
- Documentation explains how to add new endpoint validation

## Requirements

### Req 1: CLI Contract Validation Command
- 1.1: `npm run validate:contracts` command exists and works
- 1.2: Command tests against configurable daemon URL (default: http://localhost:9867)
- 1.3: Command reports pass/fail for each endpoint with clear error messages
- 1.4: Exit code 0 on success, 1 on failure, 2 on connection error

### Req 2: CI Integration
- 2.1: GitHub Actions workflow runs contract validation
- 2.2: Daemon is started as part of CI job before validation
- 2.3: CI fails if any contract mismatch is detected
- 2.4: CI artifacts include contract validation report

### Req 3: Extensibility
- 3.1: Adding new endpoint validation requires minimal code changes
- 3.2: Validation script uses centralized schema definitions (schemas.ts)
- 3.3: Documentation exists for adding new endpoint contracts

## Out of Scope

- OpenAPI spec generation (future enhancement)
- TypeShare integration improvements (separate spec)
- WebSocket message contract testing (future enhancement)
