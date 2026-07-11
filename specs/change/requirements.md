---
spec: change.spec.md
---

# Requirements

### REQ-change-001

The system SHALL require definition and closing human approval gates for every meaningful change.

#### Acceptance Criteria

- Neither gate exposes a force or emergency bypass.
- Artifact changes invalidate the definition approval digest.

### REQ-change-002

The system SHALL validate implementation against canonical contracts plus approved active semantic deltas.

#### Acceptance Criteria

- Canonical files remain unchanged before acceptance.
- Overlapping active deltas are detected before implementation.

### REQ-change-003

The system SHALL connect durable requirement IDs to technical specs, tests, and verification evidence.

#### Acceptance Criteria

- New or modified requirements use SHALL statements and acceptance criteria.
- Acceptance fails when spec-changing work has no requirement evidence.

### REQ-change-004

The system SHALL support equivalent human CLI and structured agent workflows.

#### Acceptance Criteria

- Every change operation has machine-readable JSON output.
- The same deterministic interview drives terminal and agent integrations.
