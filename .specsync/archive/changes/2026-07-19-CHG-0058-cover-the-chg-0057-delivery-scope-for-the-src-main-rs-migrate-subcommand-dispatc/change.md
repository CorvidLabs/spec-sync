---
id: CHG-0058-cover-the-chg-0057-delivery-scope-for-the-src-main-rs-migrate-subcommand-dispatc
state: archived
type: bug_fix
base_commit: 16fc94b95ce39dcdcdf9019252e6ad7eb733deef
---

# Cover the CHG-0057 delivery scope for the src/main.rs migrate subcommand dispatch and the tests/integration/change.rs CLI coverage, which were delivered under CHG-0057 but not declared in its affected paths

## Intent

Cover the CHG-0057 delivery scope for the src/main.rs migrate subcommand dispatch and the tests/integration/change.rs CLI coverage, which were delivered under CHG-0057 but not declared in its affected paths

## Affected Canonical Specs

- `cli`

## Acceptance Criteria

- src/main.rs and tests/integration/change.rs are covered by an active accepted change in delivery-diff coverage; specsync check --strict reports no uncovered meaningful changed paths; no canonical spec content changes

## No-spec Rationale

Bookkeeping delivery-scope coverage: the implementation and tests landed under accepted CHG-0057; this change only declares ownership of the two paths so delivery-diff coverage is complete
