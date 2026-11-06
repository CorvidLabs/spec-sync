# Lesson bundle: cover-the-two-paths-900-changed-outside-its-declared-scope

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Cover the two paths #900 changed outside its declared scope
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/collect_requirement_ids.rs, tests/integration/change.rs
- **Acceptance**: An approval tells me it matches the content it approved and nothing more; the name on it is a claim, not proof of who gave it.
- **Acceptance**: The change's status shows it beside the approval or review it belongs to.

## Evidence

- Verification commit: `172defb21906b81486b4202e467280d204923a79`
- Base commit: `ccc35abd6013041c37d2737e2dcb3290996f9b58`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

#900 (record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication, archived at 1a8d5048) changed two files that its affected paths did not list. On the archive tip `39d3d54d`, `change audit --strict` refused them in the CI Lifecycle gate and in Trust: "meaningful changed paths are not covered by an active change: src/change/collect_requirement_ids.rs, tests/integration/change.rs". The archived package cannot be amended, and AGENTS.md says a fix that changes the approved scope needs a new change.

- `src/change/collect_requirement_ids.rs`: `correction_prefix_digest` hashes each correction through `correction_digest_projection`, so embedded approvals' relays stay out of it. This is the Codex P1 fix in bbc46189, thread PRRT_kwDORqhp_860YaI9.
- `tests/integration/change.rs`: the exact top-level key-set test in `status_and_show_json_report_approved_delta_requirements` expects `recorded_claims` after approval and pins its value.

Leif reviewed both at 1b21d31f as part of #900. On 2026-11-06, relayed by orc t18, he said "Do the covering change": option A, with acceptance criteria CHANGE-6.d and CHANGE-13.b.

Ruled out: reverting either file (that would undo a P1 fix or fail a test) and editing the archived package.

The first cycle was approved at 4cdb2b36, verified at 31d657fd and reviewed by Leif. Then `change ship` refused: "acceptance input `src/change/collect_requirement_ids.rs` is production source without deterministic canonical ownership". With `--no-spec-change` and no affected spec, nothing owned that production file. `ship-status` had reported the change "ready to finalize". Leif said "Yes, add the spec" (2026-11-06, relayed by orc t18). The change was sent back, and `change` was added as its affected spec. Its files list owns `src/change/collect_requirement_ids.rs`, and `tests/integration/change.rs` is owned as a governed test. `--no-spec-change` stays, as in 27 archived changes.

## From the change's design.md

# Design

No code or spec text changes. The change declares the two paths so that the pull request's audit covers them:

- `src/change/collect_requirement_ids.rs`: `correction_prefix_digest` serializes `correction_digest_projection(correction)`, which clears `relayed_by` on the embedded approvals, the same projection `corrected_view_digest` uses.
- `tests/integration/change.rs`: the status JSON key set is exact. After approval it adds `recorded_claims`, which equals `[{"gate": "definition", "name": "Reviewer"}]`, and a draft omits it.

## From the change's testing.md

# Testing

The covered code already has tests from #900:

| Criterion | Test |
| --- | --- |
| CHANGE-6.d | `correction_digests_leave_embedded_relays_out` in `src/change_tests/relayed_decision.rs`: both correction digests, including `correction_prefix_digest`, equal the digests computed with the relay removed. The test fails with the projection removed. |
| CHANGE-13.b | `status_and_show_json_report_approved_delta_requirements` in `tests/integration/change.rs`: approved status JSON carries `recorded_claims` with the approval claim, the key set is otherwise exact, and a draft omits it. |

The local verify lane passed at `39d3d54d` (2838 unit, 505 integration). Before review it runs again with `change audit --strict`, and the audit must pass on the archive tip before push.

## Where these lessons go

- `specs/change/change.3md`
