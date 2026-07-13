---
change: CHG-0020-harden-reopened-acceptance-compatibility-and-canonical-governance
artifact: testing
---

# Testing

`REQ-change-020` is covered by `change::tests::reaccept_accepts_transitional_pre_reopen_definition_evidence`, `change::tests::squash_merged_acceptance_reopens_after_a_current_canonical_successor`, and `change::tests::reopened_canonical_change_validates_current_canonical_contract`.

- Reaccept accepted evidence whose audited pre-reopen verification uses the transitional explicit-false definition digest.
- Prove an overlapping accepted no-spec successor is insufficient, then prove a real semantic canonical successor is sufficient.
- Prove a malformed current canonical module blocks a reopened canonical-applied verifying record without replaying its delta.
- Run Rustfmt, Clippy with denied warnings, 1,540 unit tests, 189 integration tests, release build, and RustSec audit before closing approval. Aggregate strict SpecSync at 100% coverage, documentation/editor checks, and Trust remain configured for the final stale-evidence refresh across the accepted rollout records.
