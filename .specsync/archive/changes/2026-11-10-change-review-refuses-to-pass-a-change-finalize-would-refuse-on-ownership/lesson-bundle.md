# Lesson bundle: change-review-refuses-to-pass-a-change-finalize-would-refuse-on-ownership

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Change review refuses to pass a change finalize would refuse on ownership
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: src/change/historical_definition_digest_matches.rs, src/commands/change.rs, tests/integration/change.rs
- **Acceptance**: change review with a passing verdict refuses a verifying change whose production delivery inputs no declared module owns, naming the blocker and the send-back and answer recovery, and records nothing; a blocking verdict is still recorded; so a passing review at HEAD implies ship would not refuse on ownership, and the Archive check's change ship advice holds without a SpecSync binary

## Evidence

- Verification commit: `c8ba3f51b4a44482941ef097b2bcb6207e5761f0`
- Base commit: `6ef61e6aae6d63b2586d0f9f216c47803dba4d9f`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

Issue #1002. The Archive job, and the packaged Action's archive gate, run
`.github/scripts/archive-readiness.sh` with no SpecSync binary. It names `specsync change
ship <id>` for a change whose passing review is at HEAD. It cannot prove canonical
ownership, so it could name ship for a change ship refuses on unowned production source
(#999 fixed only the Rust side).

Leif chose option (b) on 2026-11-10 (relayed by coordinator t370): `change review` refuses
a passing verdict while `ownership_preflight_blocker` reports a blocker. A passing review at
HEAD then implies ship would not refuse on ownership, and the script's existing proof
(passing review at HEAD) is enough. A blocking verdict is still recorded.
`archive_candidate_can_ship` keeps its ownership check for reviews recorded before this.
The script is unchanged.

This also helps #1000: an over-wide scope is caught at review, before a human records a
pass, rather than only at ship.

## From the change's design.md

# Design

`record_scoped_review_with_relay` already refuses a passing verdict with open tasks, inside
`if verdict == ScopedReviewVerdict::Pass`. The ownership refusal sits in the same branch,
after the tasks check and before anything is written. It calls the same
`ownership_preflight_blocker` that ship-status, ship preflight and `change status` use, so
review, ship and finalize judge ownership identically. The error is `scoped review cannot
pass: <blocker>. Recovery: <recovery>`, reusing the blocker and recovery text ship-status
prints.

Rejected: changing `archive-readiness.sh` (option a). The script runs with no binary, so it
could only stop naming `ship` altogether. Also rejected: recording an ownership verdict in
verification for the script to read (option c), which grows the evidence schema.

## From the change's testing.md

# Testing

Unit tests in `src/commands/change.rs` and the integration test
`ship_readiness_names_unowned_production_source_before_finalize_and_the_recovery_ships`
drive approve, check, review and ship on temp git projects.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-cmd-change-008 | integration `ship_readiness_names_unowned_production_source_before_finalize_and_the_recovery_ships`; unit `an_archive_candidate_with_unowned_production_source_is_not_told_to_ship`, `the_ownership_recovery_names_a_repair_for_every_unowned_input`, `reserved_delivery_paths_are_not_ownership_blockers` |

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
