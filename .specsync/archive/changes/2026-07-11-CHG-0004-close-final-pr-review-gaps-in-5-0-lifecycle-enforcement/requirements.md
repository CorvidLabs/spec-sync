---
change: CHG-0004-close-final-pr-review-gaps-in-5-0-lifecycle-enforcement
artifact: requirements
---

# Requirements

## REQ-change-012

The lifecycle SHALL fail closed across coverage, persisted closing evidence, semantic-delta validation, dependency ordering, and supported canonical version formats.

Acceptance Criteria
- Only implementing, verifying, or accepted changes cover meaningful delivery paths.
- Local coverage includes committed, staged, unstaged, and untracked meaningful paths.
- Accepted workspaces require fresh successful verification and matching closing approval evidence.
- Delta modules, operation headings, tombstones at acceptance, and transitive dependency order are validated deterministically.
- Integer and semantic spec versions advance without losing their format.

## REQ-cmd-check-001

Unified JSON checking SHALL preserve the documented top-level check schema when SDD validation fails.

Acceptance Criteria
- Failed SDD JSON output includes `passed`, `errors`, `warnings`, `stale`, and `specs_checked`.
- Structured SDD detail remains available as an additive field.

## REQ-cmd-init-003

Fresh initialization SHALL make detected project source directories and committed SDD policy files meaningful by default.

Acceptance Criteria
- Detected source directories are merged into the generated policy.
- Policy/configuration paths cannot disable or weaken SDD coverage without lifecycle coverage.
