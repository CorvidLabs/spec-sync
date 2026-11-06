# Lesson bundle: record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Record relayed human decisions on approvals and reviews as claims, not authentication
- **Kind**: Feature
- **Specs**: change, cmd_change, cli_args
- **Paths**: src/change.rs, src/change/relayed_decision.rs, src/change/sdd_version.rs, src/change/validate_loaded_change.rs, src/change/historical_definition_digest_matches.rs, src/change_tests.rs, src/change_tests/relayed_decision.rs, src/change_tests/archive_post_move_failure_restores_exact_source_bytes_without_residue.rs, src/change_tests/substantive_plan.rs, src/cli.rs, src/commands/change.rs, tests/integration.rs, tests/integration/relayed_decision.rs, specs/change, specs/cmd_change, specs/cli_args, hi/change.md, hi/ship.md, INTENT.md, README.md, CHANGELOG.md, site/src/content/docs/workflow.md, site/src/content/docs/cli.md
- **Acceptance**: An approval tells me it matches the content it approved and nothing more; the name on it is a claim, not proof of who gave it.
- **Acceptance**: When an agent records a decision a person made somewhere else, I can see who decided, their exact words, where they said it, and which agent wrote it down.
- **Acceptance**: That record is optional, so a person approving at their own terminal is not asked for one.
- **Acceptance**: The change's status shows it beside the approval or review it belongs to.
- **Acceptance**: The status says plainly that the record is a claim too, not proof that the person decided.
- **Acceptance**: A record missing one of those details is refused before anything is written, and the refusal says which detail is missing.
- **Acceptance**: A review an agent records for a person can carry the same record, shown the same way.

## Evidence

- Verification commit: `1b21d31f60f874ac940da23b08d240330ce370a4`
- Base commit: `b179a4f254af1ddada0671eece1f8616729e2eff`
- Verified by: `specsync check --spec change --spec cli_args --spec cmd_change`

## From the change's context.md

# Context

Issue #633, audit finding D7 (`_audits/specsync-chough-20261105/REPORT.md`, `scripts/repro.py approval-identity`). An approval records whatever `--actor` it is given, and editing `actor` afterwards is accepted, because the approval digest binds content and not identity. HI CHANGE-6.b promised more than that ("A coding agent can never grant that approval to itself"). README and REQ-change-046 already called reviewer and actor labels claims.

Leif chose to narrow the promise and to record relayed decisions (95-plan row S8). On 2026-11-05 at 20:19 MST, relayed by orc t18, he approved the intent ("Approve") and said "Keep it out" on whether the relay record belongs in the review digest. The `relayed_by` shape is shared with Chough (ch-d7, chough#123); Leif approved it from ch-d7's proposal section 1.

Ruled out: any signing or authentication inside SpecSync (that is attest's job and #633's residual), and putting the relay in the review digest (6.2.x would then report every relayed archive as corrupt).

## From the change's design.md

# Design

- `RelayedDecision { decided_by, words, where, recorded_by }` (all `String`). It deserializes through a validating `TryFrom<serde_json::Value>`: the value must be an object with exactly those four string keys, each non-blank and at most 4096 bytes. The field on both records is `#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = ...)]`, so absent reads as `None` and `null` is refused.
- `relayed_decision(rerun, decided_by, words, where_, recorded_by)` builds the record from flags. With none of the three flags it returns `None`; with some, it refuses and names each missing flag plus the full re-run. `decided_by` comes from `--actor` or the validated `--reviewer`.
- `approve_definition_with_relay` passes the relay into `append_approval` and the portable pair, and refuses a relay whose `decided_by` is not the explicit actor. The old `approve_definition*` functions call it with `None`.
- `record_scoped_review_with_relay` sets `relayed_by` on the new review, and the attempt ledger keeps the same record. `scoped_review_digest(&review)` hashes the record with `relayed_by` cleared; finalization creation, finalization validation and the collapsed-archive check all use it.
- `load_approvals` and `load_scoped_review` add `git checkout <commit> -- <path>` to a refusal caused by a malformed relay record.
- `recorded_claims(root, record)` reads `approvals.json` and `review.json` wherever the change lives and returns digest-free `RecordedClaim`s. Text status prints `Approval: <actor> (a recorded claim, not authenticated)` and `Review: <reviewer> (<verdict>; a recorded claim, not authenticated)`, each followed by a `Relayed:` line when there is a relay. JSON adds `recorded_claims`, omitted when empty.

## From the change's testing.md

# Testing

CLI tests in `tests/integration/relayed_decision.rs` drive the built binary through approve, check, review and finalize. Unit tests in `src/change_tests/relayed_decision.rs` cover the record type and the review digest.

| Criterion | Test |
| --- | --- |
| CHANGE-13, REQ-change-relayed-decision-001 | a relayed approval records all four fields, with `decided_by` taken from `--actor` |
| CHANGE-13.a | an approval and a review without the flags omit `relayed_by` |
| CHANGE-13.b, 13.c, REQ-cmd-change-relayed-decision-001 | status text and JSON show the approval and review claims, the relay and the boundary sentence |
| CHANGE-13.d | one or two flags are refused naming the missing ones, nothing is written, and the named re-run succeeds; a relay without `--actor` is refused |
| SHIP-2.e | a relayed review records the relay, finalizes, and shows it in status |
| CHANGE-6.d, REQ-change-001 | status labels the approval's actor a recorded claim, not authenticated |
| Review digest ("Keep it out") | finalization's `review_digest` equals the digest of the review with `relayed_by` removed |
| Read-side refusal | a hand-edited relay (`null`, unknown key, blank) is refused naming `git checkout`, and running that command recovers |
| REQ-cli-args-relayed-decision-001 | grammar test for the three flags on approve and review |

The tests are written first and fail against `b179a4f2` (unknown flags, missing record). `fledge lanes run verify` runs before review.

Codex review of #900 (`e5716ab2`): the rerun named for an incomplete relayed review now keeps `--verdict` (and `--note`/`--portable-5-0-1` for approve); both correction digests hash embedded approvals without `relayed_by`; and a relay read back must name its record's own actor or reviewer. Tests: `a_refused_blocking_review_names_a_rerun_that_keeps_the_verdict`, `correction_digests_leave_embedded_relays_out` and `a_read_relay_must_name_its_own_record_decider`, plus a decider-mismatch case in the malformed-relay CLI test, which runs the recovery. The two unit tests fail with their fix removed. The uncommitted-`review.json` recovery gap is #904.

Second Codex round (`29a18a80`): the write APIs validate every relay field; the decider check covers every review attempt; append-only review-history comparisons ignore `relayed_by`, so an older binary's rewrite of earlier attempts is not read as removed evidence. Tests: `the_write_apis_validate_every_relay_field`, `every_review_attempt_relay_must_name_its_reviewer`, `review_history_comparisons_ignore_relays_but_not_other_edits`; all three fail with their fix removed.

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
- `specs/cli_args/cli_args.3md`
