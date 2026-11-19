# Lesson bundle: change-ship-status-exits-1-when-it-reports-a-blocker

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Change ship-status exits 1 when it reports a blocker
- **Kind**: BugFix
- **Specs**: cmd_change
- **Paths**: src/commands/change.rs, tests/integration/change.rs, site/src/content/docs/cli.md, docs/ADOPTING.md
- **Acceptance**: an approved, unverified change: change ship-status <id> prints its Blocker line and exits 1, and --json prints one document with a non-empty blockers list and exits 1; change ship-status with no id exits 1 when any listed change has a blocker; a change ready to finalize and a closed-late change exit 0 as before

## Evidence

- Verification commit: `762adcaf141279f9c483ae018634dbb4a2fead1d`
- Base commit: `18385ef5ead84f0b51d70a29a6fed8a3e5ca222a`
- Verified by: `specsync check --spec cmd_change`

## From the change's context.md

# Context

`change ship-status` prints `Blocker: …` lines when a change cannot ship yet, for example when its
verification commit is no longer an ancestor of HEAD after a squash-merge without finalize, and
still exits 0. A script or CI step that runs it reads success alongside the word Blocker (#654).
`change audit --strict` already exits 1 on stale verification evidence (re-tested by lane A on
main 18385ef5e); `ship-status` was the remaining half.

Leif's decision, 2026-11-13T14:12Z (coordinator chat, Claude Code hc t370; relayed by
claude-code-t370 and passed on by claude-code-t38c): asked 'SpecSync #654: change ship-status
prints Blocker but still exits 0. Change that?', he chose "Exit 1 on a blocker (Recommended)".

## From the change's testing.md

# Testing

- `ship_status_exits_one_when_it_reports_a_blocker` (tests/integration/change.rs): an approved,
  unverified change. Text `ship-status <id>` prints `Blocker:` and exits 1; `--json` prints one
  parseable document with a non-empty `blockers` list and exits 1; the roster form, text and
  JSON, exits 1. Red on main: every form exits 0.
- Three existing tests that ran ship-status on a blocked change through a success-asserting
  helper now assert exit 1 and read the same report:
  `a_change_reviewed_before_its_send_back_is_reviewed_again_and_finalizes`,
  `a_send_back_needs_a_fresh_review_even_when_the_scope_comes_back_unchanged`,
  `ship_readiness_names_unowned_production_source_before_finalize_and_the_recovery_ships`.
- Ready-to-finalize ship-status still exits 0 (the last test's `ready` read asserts success).

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-cmd-change-039` | `ship_status_exits_one_when_it_reports_a_blocker` in `tests/integration/change.rs` (text, JSON and roster exit 1 on a blocker; JSON is one document); `ship_readiness_names_unowned_production_source_before_finalize_and_the_recovery_ships` (a ready change exits 0) |

## Where these lessons go

- `specs/cmd_change/cmd_change.3md`
