# Lesson bundle: retain-compound-historical-review-inconsistencies-during-anchored-reopen

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Retain compound historical review inconsistencies during anchored reopen
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/historical_definition_digest_matches.rs, src/change_tests/recorded_review_recovery.rs, specs/change, CHANGELOG.md
- **Acceptance**: Within exact recorded workspace-mismatch recovery, preserve the entire historical review including its inconsistent claimed hash as untrusted; keep every normal validation and non-review digest strict, refuse a pure review-hash mismatch, require exact remote-default package anchoring and fresh verification/review, and qualify the public source against the tested implementation.

## Evidence

- Verification commit: `d9fd9401a4eb99ff960de257819307b305848a1e`
- Base commit: `9eba6ed1a9f24226fe24f08d890bbbd70245dbb5`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

A recorded historical review can disagree with both the accepted workspace and the claimed review hash. Reopen preserves that whole invalid review as untrusted evidence; normal acceptance and archive still refuse it. The exact remote-default package and all non-review integrity checks remain mandatory.

## From the change's design.md

# Design

Only the existing reopen-only recorded workspace-mismatch mode retains the old review hash as untrusted. Its package is first compared byte-for-byte and mode-for-mode with the remote-default ancestor tree. Definition, execution, closing, implementation tree and finalization hashes remain strict. The old review never gains acceptance authority; closure needs fresh verification and scoped review.

## From the change's testing.md

# Testing

## Acceptance Criteria

REQ-change-117: the compound discriminator failed before the fix; all four focused recovery methods pass afterward, including the pure-hash and finalization-digest refusal controls. The qualified source passed the full pinned-Rust lane: 2691 unit tests, 478 integration tests, 66 release-contract tests, 35 CI-gate tests, strict 62-module checks and 100 percent coverage. Branch source equality and exact-head hosted checks are separately verified.

All 146 Rust/build input files match the fully qualified source. SHA-256 of sorted path-to-SHA256 JSON: `3ecd8f53060037d3b713531a4d1d61eedc06899a4c9488f2160a1a8041438cdd`. Current-main release/CI workflow contract tests passed separately after the workflow-only base update.

## Where these lessons go

- `specs/change/change.3md`
