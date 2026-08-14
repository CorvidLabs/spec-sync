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

The `generator` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

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


### REQ-generator-003

Generated output SHALL NOT embed a coverage percentage that was not measured.

Acceptance Criteria
- A zero denominator renders the unmeasured state.
