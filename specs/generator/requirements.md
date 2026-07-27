---
spec: generator.spec.md
---

## User Stories

- As a developer, I want reproducible local scaffolds for every uncovered module.
- As a team, I want custom templates and existing files preserved.
- As a coding-agent user, I want standard companion files ready for enrichment.

## Constraints

- Generated files must be valid inputs to `specsync check`.
- Source discovery excludes tests and remains deterministic.

### REQ-generator-001

The generator SHALL produce deterministic local scaffolds that a human or connected coding agent can refine.

Acceptance Criteria
- Generation has no provider parameter or AI error channel.
- Custom and language-aware templates, source detection, companions, and no-overwrite guarantees remain unchanged.

### REQ-generator-002

The CLI generator SHALL keep every project filesystem side effect beneath one retained
project-root capability.

Acceptance Criteria

- Template reads, module-directory creation, spec publication, and companion publication are
  relative to the retained capability.
- Configured specs paths and module destinations reject absolute, rooted, prefix, and parent
  traversal components before use.
- Redirecting the caller-visible root after checked coverage cannot redirect an output write.
- Existing files remain no-overwrite destinations.

