# Lesson bundle: keep-the-upstream-archive-status-guidance-behind-a-sequence-ledger-freeze

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep the upstream-archive status guidance behind a sequence-ledger freeze
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: src/change/, src/commands/change.rs, tests/integration/archived_upstream.rs, CHANGELOG.md
- **Acceptance**: When a change is archived upstream and the change-sequence ledger is frozen, `change status --json` gives the freeze in `next_action` and in `handoff.before_clearing`, not the upstream-archive rebase step.
- **Acceptance**: Text `change status` and the ship-status lifecycle next action give the same freeze step in that case.
- **Acceptance**: Without a freeze, the upstream-archive guidance is unchanged: text, JSON next_action and handoff all name it.
- **Acceptance**: An integration test built from the auditor's squash fixture plus a frozen ledger fails on main first and passes after.

## Evidence

- Verification commit: `0373671e6638ad8d0e7e30f274b459df14d4d4fa`
- Base commit: `cd8cab5e1789246f6f3a4bf6733b33d301d3ca4b`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

spec-sync#908, a review follow-up from #889. With an upstream archive and a frozen change-sequence ledger, JSON `next_action` names the freeze (it is checked first), but the status summary overrides the handoff unconditionally with `upstream_archive_handoff`, so `handoff.before_clearing` tells the operator to rebase. Rebasing does not clear a project-wide freeze.

Reproduced on main cd8cab5e by `a_sequence_ledger_freeze_outranks_the_upstream_archive_guidance`: the handoff leads with "already archived on `origin/main`" while `next_action` names the sequence ledger.

## From the change's design.md

# Design

One priority order for every surface: sequence-ledger freeze, then the verification_ignored_paths refusal, then the upstream archive, then state-local steps.

- Status summary (`src/change/historical_definition_digest_matches.rs`): use `upstream_archive_handoff` only when neither the freeze nor the verification_ignored refusal applies; otherwise `classify_handoff` already reports them.
- Text next action (`text_mode_next_action`, `src/commands/change.rs`, also ship-status's lifecycle next): return the freeze remediation before the upstream archive.
- `handoff_summary` already returns the freeze before checking the upstream archive.

## From the change's testing.md

# Testing

`tests/integration/archived_upstream.rs::a_sequence_ledger_freeze_outranks_the_upstream_archive_guidance`: the auditor's squash fixture with `origin/main` holding the archive, then an acknowledgement in `.specsync/change-sequence.json` that freezes the ledger. Asserts JSON `next_action` and the first `handoff.before_clearing` entry name the sequence ledger, not the upstream archive, the text `Next:` line names the sequence ledger, and `change ship-status` names the freeze before the upstream archive. On main cd8cab5e the handoff assertion fails.

The existing `archived_upstream` tests (no freeze) stay green, covering the unchanged guidance.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-138 | `a_sequence_ledger_freeze_outranks_the_upstream_archive_guidance` (JSON `next_action` and `handoff.before_clearing` name the freeze), `a_verification_ignored_paths_refusal_reaches_the_handoff_of_an_upstream_archived_change` (the handoff carries the ignored-path refusal, `not-yet`; failed first at 1f625f35), `a_change_already_archived_on_the_remote_default_is_named_and_refused_before_acceptance` (without a freeze both name the upstream archive) |
| REQ-cmd-change-034 | `a_sequence_ledger_freeze_outranks_the_upstream_archive_guidance` (text `Next:` and `change ship-status` name the freeze first) |

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
