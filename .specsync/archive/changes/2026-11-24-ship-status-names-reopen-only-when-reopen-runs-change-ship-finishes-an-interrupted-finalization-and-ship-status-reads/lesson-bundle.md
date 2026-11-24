# Lesson bundle: ship-status-names-reopen-only-when-reopen-runs-change-ship-finishes-an-interrupted-finalization-and-ship-status-reads

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Ship-status names reopen only when reopen runs, change ship finishes an interrupted finalization, and ship-status reads the ledger freeze once per report
- **Kind**: BugFix
- **Specs**: cmd_change, change
- **Paths**: src/commands/change.rs, src/change/consider_accepted_evidence_anchor.rs, src/change/historical_definition_digest_matches.rs, src/change_tests/historical_reconciliation.rs, tests/integration.rs, tests/integration/ship_status_reopen_and_resume.rs
- **Acceptance**: #922: an archived change whose own accepted evidence is current is not told to reopen after a commit that changes only an unrelated file, its product-tip stage is done, and `change reopen` refuses it; an accepted or archived change whose own accepted evidence is stale is told to reopen in the product-tip stage and in `Next:`, never to run `change check`, and `change reopen` accepts it; an archived change whose `verification.json` cannot be read is told to restore it and is not told to reopen.
- **Acceptance**: #933: after a finalization that stopped between acceptance and the archive move, `change ship-status <id>` reports the accepted change ready to finalize and `change ship <id>` archives it; with a task reopened in that state, `change ship <id>` refuses naming the task and `specsync change ship <id>`, and once the task is ticked that named command archives the change; an accepted change that `change finalize` would refuse to resume is not reported ready.
- **Acceptance**: #938: `change ship-status` evaluates the sequence-ledger freeze once for each change it reports, and its blockers, `ship_next` and lifecycle next action are unchanged, with and without a freeze.
- **Acceptance**: #1038 follow-up: the reconciled-history test asserts that an archived change whose terminal evidence is `historical-transition-unproven` reads its review as done by finalize.

## Evidence

- Verification commit: `6ccac2b26d7cd2b4b1b1a576ce26dfc79a4fa572`
- Base commit: `a28e30420c12627a384a685cd5d808fc3585b70a`
- Verified by: `specsync check --spec change --spec cmd_change --strict`

## From the change's context.md

# Context

- **#922** (the issue body and Leif's 2026-11-06 comment): ship-status's product-tip stage names `change reopen` for an accepted or archived change whenever whole-project verification is not current. `change reopen` judges only the change's own accepted evidence, so after a commit that changes an unrelated file it refuses ("delivery inputs are current"), and with an unreadable `verification.json` it fails ("invalid verification evidence"). For an archived change with stale evidence, `ship_next` says `change check <id> --commit`, which refuses an archived change.
- **#933** (Codex P2 on #903): REQ-cmd-change-031's open-task refusal names `specsync change ship <id>`, and text status tells an accepted workflow-v2 change to run `change ship` or `change finalize`; ship readiness accepts only a verifying change, so after a finalization that stopped between acceptance and the archive move, the named command refuses.
- **#938** (Codex P2 on #934): ship-status evaluates the sequence-ledger freeze twice for each report.
- **#1038 follow-up**: ss-lead, 2026-11-19, the assertion for the `historical-transition-unproven` arm goes into this change.
- The spike on 4d439b39 reproduced all three #922 cases; `validate_accepted_inputs_recursive` agreed with `change reopen` in each. The unapproved 11-13 branch `fix/922-product-stage-reopen-only-when-stale` mapped terminal evidence to a state, where an archived change never reads stale; its fix is redone and its two failing tests are kept.
- Leif's decision, 2026-11-18, relayed by hc t4c through ss-lead (hc t58): fix #938 with the ship-status batch, together with #922 and #933, as a new change with its own definition. #933 takes the issue's option 1.

## From the change's design.md

# Design

- **#922, asked of the change's own evidence.** For an accepted or archived change, ship-status asks the validator that already judges accepted and archived inputs, read-only, whether the change's own accepted evidence is current. Current: the product-tip stage is done. Stale: the product-tip stage and `ship_next` name `specsync change reopen <id> --actor <name> --reason <text>`, never `change check`. `verification.json` unreadable: both name its restore, not reopen.
- **#933, readiness as finalize judges it.** An accepted change is ready to finalize when `change finalize` would resume it: the check finalize itself makes before it resumes (its `finalization.json` and the review it binds), with no blocker. `change ship` then archives it through finalize's resume. An open task still blocks, naming the task and `specsync change ship <id>`.
- **#938, once per report.** `ship_status_report` evaluates the sequence-ledger freeze once and passes it to the lifecycle next action, so the blockers, `ship_next` and the lifecycle next action read the same value.
- **#1038.** One assertion in the reconciled-history test: an archived change whose terminal evidence is `historical-transition-unproven` reads its review as done by finalize.

Rejected: #933's option 2 (name `change finalize` and correct AGENTS.md), because AGENTS.md, REQ-cmd-change-031 and text status already promise that `change ship` works there.

## From the change's testing.md

# Testing

- **Unit, failing first** (`src/commands/change.rs`): the two kept #922 tests (an archived change not told to reopen for unrelated drift; an unreadable `verification.json` not told to reopen), plus stale accepted and archived changes told to reopen in the product-tip stage and `ship_next`, never `change check`; #933 on a real interrupted finalization (a test-only stop after acceptance), ready and archived by `change ship`, an open task named with `specsync change ship <id>`, and an accepted change finalize would not resume not reported ready; #938 counting freeze evaluations per report, with and without a freeze.
- **Through the binary**: at least one test per fix, `change ship-status` and `change ship` as a person runs them.
- **Agreement**: in each #922 case, `change reopen` refuses or accepts as ship-status says.
- **#1038**: the assertion in `src/change_tests/historical_reconciliation.rs`.
- **Gates**: `change audit --strict` with origin/main fetched, and `fledge lanes run verify`.

## Where these lessons go

- `specs/cmd_change/cmd_change.3md`
- `specs/change/change.3md`
