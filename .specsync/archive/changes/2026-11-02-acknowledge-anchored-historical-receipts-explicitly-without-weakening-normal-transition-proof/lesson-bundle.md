# Lesson bundle: acknowledge-anchored-historical-receipts-explicitly-without-weakening-normal-transition-proof

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Acknowledge anchored historical receipts explicitly without weakening normal transition proof
- **Kind**: Feature
- **Specs**: change, cmd_change, cli_args
- **Paths**: src/change, src/change.rs, src/commands/change.rs, src/cli.rs, src/change_tests, src/change_tests.rs, hi, site/src/content/docs/cli.md, site/src/content/docs/workflow.md, CHANGELOG.md
- **Acceptance**: Opt-in historical reconciliation preserves authenticated committed predecessor receipts, labels old transitions unproven, remains bound to exact path/module/digest obligations, and requires newly approved scope plus fresh current verification and scoped review before closure. Normal before/after succession stays strict.

## Evidence

- Verification commit: `5cc27e1a07242b20dd68ce5c08b4a89235fcb8f4`
- Base commit: `0b06c18aa2b6d6549f0bdb1e221b7a17e284c099`
- Verified by: `specsync check --spec change --spec cli_args --spec cmd_change`

## From the change's context.md

# Context

A present-day definition can hold the authentic old signed receipt while its base files have already drifted. Strict before/after succession correctly refuses it. The approved new feature records an explicitly weaker historical acknowledgement, preserves the complete old package at its signed-base Git anchor, authenticates its existing normal acceptance, and verifies only the new current receipt as current delivery evidence. No old file transition or downstream product acceptance is invented.

## From the change's design.md

# Design

`change reconcile-history ID --actor --reason` requests a new definition-bound mode on an unaccepted v2 successor with existing exact obligations. The immutable existing base is the anchor; the command never approves or closes it. Fresh scope approval is mandatory. Historical schema-2 evidence binds sorted predecessor package and manifest digests, explicitly marks transition status unproven, and still proves each successor input at its current acceptance anchor. Original active accepted bytes must equal the committed anchored package; native archives may normalize only their authenticated accepted-state snapshot. Only regular file, missing and directory entries outside protected tool metadata are supported. Current ownership and every ordinary signature/digest/cycle/ancestry/resource check remain enforced.

## From the change's testing.md

# Testing

## Acceptance Criteria

Use a genuine legacy receipt whose files change before creating its successor: normal succession must still refuse, while only a newly approved historical request can close. Verify exact old package preservation, explicit schema/status, and fresh verification/review requirements. Negative controls cover missing or non-ancestor anchors, non-accepted or tampered packages/receipts, forged witness digests, false proven claims, stale successor bytes, protected paths and unsupported entry types. Run pinned Rust 1.89 full Fledge verification, strict specs/full configured coverage, pre-push, Augur, Attest and Trust; qualify the real consumer separately.


## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-118 | historical_reconciliation_* lifecycle and negative controls in src/change_tests/historical_reconciliation.rs |
| REQ-cmd-change-025 | Actual consumer CLI status/finalize JSON inspection and unchanged normal output controls |
| REQ-cli-args-020 | change_historical_reconciliation_requires_explicit_actor_and_reason |


Executed qualification: six targeted checks and the complete Fledge verify lane passed (2,697 unit, 478 integration, 66 release-contract and 35 CI-contract tests) with pinned Rust 1.89. The isolated missing-digest-binding mutation failed the old-approval assertion. Rust/build-input equality was checked after the separately landed bounded recovery merged into main. Documentation and actual consumer archival results are recorded separately as they complete. No human code review or published release is claimed.

Real-consumer performance follow-up: reuse authenticated successor evidence, immutable predecessor witnesses and acceptance anchors only within one pure graph traversal. Discard the memo before every lifecycle write and create a fresh one for each pass. The lifecycle discriminator measures fewer Git reads on a repeated tuple and proves a new pass refuses altered raw receipt bytes even when JSON semantics are unchanged. All original bounds, proof checks and serialized evidence remain unchanged. Full qualification is rerun after this production-code refinement.

Native archival regenerates change.md as well as writing the accepted-state preimage. A historical witness after native archival must remain exactly equal to the original witness. Only markdown equal to the native rendering of the authenticated archived record may be normalized back to its immutable original bytes; a one-byte markdown alteration remains refused. Full qualification is rerun after this refinement.

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
- `specs/cli_args/cli_args.3md`
