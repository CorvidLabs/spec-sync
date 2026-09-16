---
id: add-a-hi-check-gate-to-ci
state: implementing
type: operations
base_commit: 64abf58182ab222d2415d7ffcffabc9b0c8ed151
---

# Add a hi check gate to CI

## Intent

Add a hi check gate to CI

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- A full CI run executes hi check over hi/*.md and fails the job when a file is structurally broken (duplicate id, case with no parent, id colliding with a retired one, malformed id, undeclared family, stranded criterion). A pull request touching only hi/** triggers ci.yml and reaches the required gate rather than waiting forever on a status that is never reported. The hi-check job is a dependency of implementation-gate, so a structural break blocks merge. The installed binary is human-intent pinned to 0.4.0 with --locked.

## No-spec Rationale

The change adds a CI step that installs a pinned external binary and runs it; no canonical spec behaviour changes, and specs/github already owns the workflow files.
