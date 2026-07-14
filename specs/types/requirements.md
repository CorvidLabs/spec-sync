---
spec: types.spec.md
---

## User Stories

- As a contributor, I want shared deterministic types in one dependency-light module.
- As a language maintainer, I want consistent extension and test-pattern mapping.
- As an integrator, I want stable validation and coverage result structures.

## Constraints

- The module has no dependency on other SpecSync modules.
- Default values are deterministic, local, and usable.
- MSRV remains 1.89.

### REQ-types-001

Core shared types SHALL contain no embedded inference-provider or credential configuration.

Acceptance Criteria
- `AiProvider` and its helper API are removed.
- `SpecSyncConfig` has no AI provider, model, command, key, base URL, or timeout fields.

### REQ-types-002

The shared language registry SHALL classify `.mjs` and `.cjs` as TypeScript-family sources for both direct detection and default extension discovery.

Acceptance Criteria

- Direct extension lookup maps `mjs` and `cjs` to TypeScript.
- The TypeScript default extension list includes `mjs` and `cjs` alongside existing JavaScript and TypeScript suffixes.
- Explicit source-extension filtering remains unchanged.

### REQ-types-003

Shared validation types SHALL represent planned draft mappings as notices distinct from errors and warnings.

Acceptance Criteria

- Notices carry deterministic human-readable messages.
- Notices do not increment warning counts or fail strict validation.
- The shared configuration type exposes the default-false draft-file enforcement setting.

