---
id: cover-the-integration-fixtures-the-ordinal-retirement-rewrote
state: implementing
type: operations
base_commit: 36041a548e31d6999df5a64df273a3e6da530b0c
---

# Cover the integration fixtures the ordinal retirement rewrote

## Intent

cover the integration fixtures the ordinal retirement rewrote

## Affected Canonical Specs

- None

## Acceptance Criteria

- The ordinal retirement rewrote assertions in tests/integration/change.rs and tests/integration/comment.rs that hard-coded the CHG-NNNN identity shape, but those two paths were not in its declared scope, so the workspace has meaningful changed paths no active change covers. Done when both files are covered by an accepted change and audit --strict is clean.

## No-spec Rationale

Integration fixtures asserting the old CHG-NNNN identity shape; the behaviour they cover is specified by REQ-change-086, which the retirement change already added
