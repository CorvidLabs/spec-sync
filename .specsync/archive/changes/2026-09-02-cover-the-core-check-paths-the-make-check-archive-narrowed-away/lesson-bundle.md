# Lesson bundle — cover-the-core-check-paths-the-make-check-archive-narrowed-away

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Cover the core-check paths the make-check archive narrowed away
- **Kind**: Operations
- **Specs**: cmd_check, cmd_init
- **Paths**: site/src/content/docs/deltas.md, src/commands/check.rs, src/commands/init.rs, tests/integration/commands.rs
- **Acceptance**: The core-check cut (84cb5cae, 359eeee2) changed src/commands/check.rs, src/commands/init.rs, tests/integration/commands.rs and site/src/content/docs/deltas.md, but the archived make-check-the-product record no longer lists them: its affected_paths were narrowed from directory scopes to explicit files so finalize could build an acceptance manifest, and these four were dropped. Done when all four paths are covered by an accepted change on this branch, specsync change audit --strict exits 0 on the branch tip with no active change, and specsync check --strict still passes with no spec text change.

## Evidence

- Verification commit: `fe891f7abd9d98bfd6ed1b52cb4fbf8f959d83e3`
- Base commit: `ea838cc3fc06c4cc528709051a60a4d03aaf5b3b`
- Verified by: `specsync check --spec cmd_check --spec cmd_init`

## From the change's context.md

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

## From the change's testing.md

# Testing

No new assertions: this record changes no file. The paths it covers are already exercised —
`tests/integration/commands.rs` is run by `cargo test --test integration` (409 integration
tests, 0 failures, on this branch; `full-test.log`), and the `cmd_check` / `cmd_init` specs
that own the two production files pass `specsync check --strict` (62/62).

The gate this record exists for is `specsync change audit --strict`: it exits 1 on the branch
tip before this record and must exit 0 after the record is archived. That command is what
`.github/workflows/trust.yml` runs, so the same verdict lands in CI.

## Where these lessons go

- `specs/cmd_check/context.md`
- `specs/cmd_init/context.md`
