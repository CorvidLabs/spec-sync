---
change: keep-receipt-reads-bounded-and-recover-exactly-recorded-review-mismatches
artifact: testing
---

# Testing

## Acceptance Criteria

REQ-change-117 is covered by recorded_review_recovery_preserves_preimage_and_requires_fresh_check_and_review, recorded_review_recovery_refuses_unanchored_or_modified_packages_without_writes and recorded_review_recovery_does_not_relax_other_finalization_digests. Frozen read scope is covered by frozen_manifest_ignores_later_oversized_assets_but_full_inventory_stays_bounded. The full qualified source passed 2690 unit tests, 478 integration tests, 66 release-contract tests, 35 CI-gate tests, strict 62-module checks and 100 percent coverage. This branch source equality and exact-head hosted checks are recorded separately rather than presumed.

Publication source equals the fully qualified source across 146 Rust/build input files. SHA-256 of sorted path-to-SHA256 JSON: `5a688e2071f5bb6353b6c09bee2cf5440ca8212127df61337822497e9371d4d7`. Metadata and native lifecycle checks on this branch remain independent gates.
