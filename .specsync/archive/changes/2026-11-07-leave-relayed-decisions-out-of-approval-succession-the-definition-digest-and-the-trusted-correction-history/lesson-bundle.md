# Lesson bundle: leave-relayed-decisions-out-of-approval-succession-the-definition-digest-and-the-trusted-correction-history

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Leave relayed decisions out of approval succession, the definition digest and the trusted correction history
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/relayed_decision.rs, src/change/consider_accepted_evidence_anchor.rs, src/change/collect_requirement_ids.rs, src/change/historical_definition_digest_matches.rs, src/change_tests/relayed_decision.rs, specs/change, hi/change.md, INTENT.md, CHANGELOG.md
- **Acceptance**: One of those records never changes what an approval, review or check decides, so an older SpecSync that drops it can still reopen and finish the change.

## Evidence

- Verification commit: `b87cb25ea6f2bf6a484083525288f19a808f8463`
- Base commit: `71130737616cf0379694f5e7b0446bceff4628e7`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

#900 made a relayed decision (`relayed_by`) a claim that no digest covers and no gate reads, so an older 6.x binary that drops the unknown key computes the same answers ("Keep it out", Leif, 2026-11-05). The review digest, both correction digests and the review-history comparisons already leave it out. Codex found three comparisons that still see it:

- #926: the approval-succession check of a reopened package (`ledger_succession`) compares approvals and reopen events as raw JSON, so an older binary's reopen, which rewrites `approvals.json` without the key, is not a lawful successor and finalization is blocked.
- #927: the definition digest hashes the correction ledger as written, so a relay embedded in a correction's superseded approvals is in the digest the approval binds and the execution digest reaches; editing a relay changes gate evidence, and an older binary computes a different digest.
- #971: the trusted correction-history comparison compares whole corrections, so an older binary's append reads as divergence.

Leif captured CHANGE-13.h on 2026-11-06 (relayed by orc t18, "One sentence"). The orc assigned REQ-change-relay-compare-001.

The failing tests are committed first (fe9615c1). Ruled out: keeping the raw comparisons and documenting the limit (it leaves CHANGE-13.h false), and weakening any of the three checks beyond the relay (every other field is still compared exactly).

## From the change's design.md

# Design

- `ledger_succession`: compare approvals with `relayed_by` cleared on each, reopen events with `relayed_by` cleared on their `superseded_approval`, and the superseded approval with the earlier terminal approval the same way. A small helper projects the JSON values; every other key is compared as before.
- The definition digest hashes the correction ledger through `correction_digest_projection`, the projection both correction digests use. The historical definition digest, which authenticates a committed correction as a trusted anchor, hashes it the same way; otherwise a relayed correction would stop being a trusted anchor.
- `validate_trusted_correction_history` compares `correction_digest_projection` of each trusted correction with that of the current one.
- No new public API; the three call sites reuse the existing projection.

## From the change's testing.md

# Testing

Unit tests in `src/change_tests/relayed_decision.rs`, written first (fe9615c1); each failed on `main` b3ca5f97.

| Criterion | Test |
| --- | --- |
| CHANGE-13.h, REQ-change-relay-compare-001 (#926) | `approval_succession_ignores_a_relay_an_older_binary_dropped`: an older binary's reopen without `relayed_by` is a lawful successor; an edited approval or superseded approval is not |
| CHANGE-13.h, REQ-change-relay-compare-001 (#927) | `the_definition_digest_leaves_relays_in_corrections_out`: the digest is the same with a correction's relays present, edited or removed, and changes for another edit |
| CHANGE-13.h, REQ-change-relay-compare-001 (#971) | `trusted_correction_history_ignores_a_relay_an_older_binary_dropped`: a ledger rewritten without the embedded relay is the same history; an edited reason is divergence |

`fledge lanes run verify` runs before review.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-relay-compare-001 | unit tests approval_succession_ignores_a_relay_an_older_binary_dropped, the_definition_digest_leaves_relays_in_corrections_out and trusted_correction_history_ignores_a_relay_an_older_binary_dropped |

## Where these lessons go

- `specs/change/change.3md`
