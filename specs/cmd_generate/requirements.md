---
spec: cmd_generate.spec.md
---

## User Stories

- As a developer, I want deterministic local scaffolds for uncovered modules.
- As a developer, I want batch generation for selected modules.
- As an agent integrator, I want stable JSON paths for later refinement.

## Constraints

- Existing specs are never overwritten.
- Agent integrations may refine generated markdown outside this command.

### REQ-cmd-generate-001

The generate command SHALL create deterministic local specs only from trustworthy discovery.

Acceptance Criteria

- All generation modes use checked coverage discovery before selecting output.
- Malformed Gradle/manifest discovery exits nonzero before mutation.
- JSON mode remains parseable with `valid: false`, `inconclusive: true`, an explicit error, and an
  empty `generated` collection.

