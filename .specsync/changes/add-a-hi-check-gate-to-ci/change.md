---
id: add-a-hi-check-gate-to-ci
state: verifying
type: operations
base_commit: 64abf58182ab222d2415d7ffcffabc9b0c8ed151
---

# Add a hi check gate to CI

## Intent

Add a hi check gate to CI

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- A full CI run executes hi check over hi/*.md and fails the job when a file is structurally broken (duplicate id, case with no parent, id colliding with a retired one, malformed id, undeclared family, stranded criterion). A pull request touching only hi/** triggers ci.yml and reaches the required gate rather than waiting forever on a status that is never reported. The hi-check job is a dependency of implementation-gate and of the corvid-pet scoped-review job, so a structural break blocks merge and cannot post a passing review. The installed binary is human-intent pinned to 0.5.0 with --locked. Cargo.lock pins rustls at >=0.23.45 so cargo audit is not red on RUSTSEC-2026-0285.

## No-spec Rationale

The change adds a CI step that installs a pinned external binary and runs it, and bumps a transitive TLS crate in the lockfile so the existing audit job can pass. No canonical spec behaviour changes, and specs/github already owns the workflow files.
