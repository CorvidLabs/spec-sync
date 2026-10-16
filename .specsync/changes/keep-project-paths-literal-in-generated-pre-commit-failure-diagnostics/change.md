---
id: keep-project-paths-literal-in-generated-pre-commit-failure-diagnostics
state: implementing
type: bug_fix
base_commit: 621a74785e3c09b2e9ebea35768e05f5772f886f
---

# Keep project paths literal in generated pre-commit failure diagnostics

## Intent

Keep project paths literal in generated pre-commit failure diagnostics

## Affected Canonical Specs

- `hooks`

## Acceptance Criteria

- Newly generated hooks pass the exact project root to the checker and preserve success/failure exit behavior for ordinary paths, spaces and apostrophes; dollar substitutions and backticks never execute from failed-check diagnostics; executable negative controls fail on released 6.1.1 and pass on the candidate; existing-hook refresh instructions state that install alone does not replace managed blocks.

## No-spec Rationale

Not applicable
