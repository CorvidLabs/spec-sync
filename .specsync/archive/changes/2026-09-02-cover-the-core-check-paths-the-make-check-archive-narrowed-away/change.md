---
id: cover-the-core-check-paths-the-make-check-archive-narrowed-away
state: archived
type: operations
base_commit: ea838cc3fc06c4cc528709051a60a4d03aaf5b3b
---

# Cover the core-check paths the make-check archive narrowed away

## Intent

cover the core-check paths the make-check archive narrowed away

## Affected Canonical Specs

- `cmd_check`
- `cmd_init`

## Acceptance Criteria

- The core-check cut (84cb5cae, 359eeee2) changed src/commands/check.rs, src/commands/init.rs, tests/integration/commands.rs and site/src/content/docs/deltas.md, but the archived make-check-the-product record no longer lists them: its affected_paths were narrowed from directory scopes to explicit files so finalize could build an acceptance manifest, and these four were dropped. Done when all four paths are covered by an accepted change on this branch, specsync change audit --strict exits 0 on the branch tip with no active change, and specsync check --strict still passes with no spec text change.

## No-spec Rationale

The behaviour these four paths implement (check is the product and needs no SDD policy; init writes sdd.json with enabled:false and points at change adopt; the deltas guide) was specified and materialized by the archived make-check-the-product change. That record's affected_paths were narrowed to explicit files for finalize's acceptance manifest, which dropped these already-committed paths from coverage. No canonical spec text changes.
