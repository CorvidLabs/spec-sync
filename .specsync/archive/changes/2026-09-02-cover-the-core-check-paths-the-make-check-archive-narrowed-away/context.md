---
change: cover-the-core-check-paths-the-make-check-archive-narrowed-away
artifact: context
---

# Context

PR 748 made `specsync check` the product and put SDD behind `specsync change adopt`. Its two
core commits (84cb5cae, 359eeee2) edited `src/commands/check.rs`, `src/commands/init.rs`,
`tests/integration/commands.rs` and `site/src/content/docs/deltas.md`, and the covering record
`make-check-the-product-and-stop-change-check-from-spawning-project-tests` declared directory
scopes (`src/`, `tests/`, `site/`, …) that covered them. CI's `change audit --strict` passed on
that basis while the record was active.

Finalize refused the directory scopes: the acceptance manifest needs file-level
`affected_paths` with deterministic canonical ownership. The scopes were narrowed to explicit
files so the record could be accepted and archived — and these four paths were left out of the
narrowed list. Once the record was archived, nothing on the branch covered them, and
`change audit --strict` (the exact command `.github/workflows/trust.yml` runs) fails on the tip
with "meaningful changed paths are not covered by an active change".

Ruled out: editing the archived record (breaks its digests), `reopen` (moves Accepted, not
Archived, records), and declaring a covering prefix such as `--path src/` here (would trip the
same finalize manifest rule again). The audit's own hint — a no-spec-change record naming the
four paths — is the remedy, and the repository has used it before
(`cover-the-integration-fixtures-the-ordinal-retirement-rewrote`, CHG-0058, CHG-0122).

Nothing in these files changes under this record; every edit it covers is already committed.
