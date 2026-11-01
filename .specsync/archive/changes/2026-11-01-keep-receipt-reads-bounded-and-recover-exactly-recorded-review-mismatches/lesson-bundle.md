# Lesson bundle: keep-receipt-reads-bounded-and-recover-exactly-recorded-review-mismatches

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep receipt reads bounded and recover exactly recorded review mismatches
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/collect_requirement_ids.rs, src/change/consider_accepted_evidence_anchor.rs, src/change/historical_definition_digest_matches.rs, src/change/sdd_version.rs, src/change_tests.rs, src/change_tests/substantive_plan.rs, src/change_tests/recorded_review_recovery.rs, specs/change, hi/recover.md, INTENT.md
- **Acceptance**: Restrict recursive frozen-receipt reads to signed entries while retaining full fresh/reopen discovery and all bounds; admit reopen-only recovery of an exactly remote-default recorded active v2 review-workspace mismatch without trusting it; preserve original bytes through an ancestor Git anchor, require fresh verification and review, reject tampering and other invalid digests without writes, and retain backward-compatible optional audit metadata.

## Evidence

- Verification commit: `ce4e4ec073a8c3c23f1bc1255877d5bc372bd8bd`
- Base commit: `3d3d6e6db88077b1c233b37e4958916d8da40bbd`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Two acceptance-evidence failures can block archival: later assets can exhaust an old receipt read even though they were not signed inputs; a historically recorded finalization can have a review-workspace mismatch that cannot be reopened for fresh review. This change preserves strict acceptance and archival while giving only the exactly recorded invalid review a bounded downgrade path.

## From the change's design.md

# Design

Recursive accepted-input validation captures exact signed paths. Fresh acceptance and reopen retain full inventory and unchanged limits. A reopen-only fallback admits only an active v2 package whose raw files and modes equal the current remote-default tree, whose commit is an ancestor of HEAD, and whose other definition/execution/closing/review hashes remain valid. The optional audit anchor preserves the original package; the old mismatched review remains invalid. No serialized enum value is added.

## From the change's testing.md

# Testing

## Acceptance Criteria

REQ-change-117 is covered by recorded_review_recovery_preserves_preimage_and_requires_fresh_check_and_review, recorded_review_recovery_refuses_unanchored_or_modified_packages_without_writes and recorded_review_recovery_does_not_relax_other_finalization_digests. Frozen read scope is covered by frozen_manifest_ignores_later_oversized_assets_but_full_inventory_stays_bounded. The full qualified source passed 2690 unit tests, 478 integration tests, 66 release-contract tests, 35 CI-gate tests, strict 62-module checks and 100 percent coverage. This branch source equality and exact-head hosted checks are recorded separately rather than presumed.

Publication source equals the fully qualified source across 146 Rust/build input files. SHA-256 of sorted path-to-SHA256 JSON: `5a688e2071f5bb6353b6c09bee2cf5440ca8212127df61337822497e9371d4d7`. Metadata and native lifecycle checks on this branch remain independent gates.

## Where these lessons go

- `specs/change/change.3md`
