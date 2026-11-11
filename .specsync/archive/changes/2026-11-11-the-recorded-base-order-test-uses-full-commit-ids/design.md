---
change: the-recorded-base-order-test-uses-full-commit-ids
artifact: design
---

# Design

- `recorded_diff_base` considers only the oldest active change's `base_commit`, and uses it only when it
  is a full commit id. Any other value falls back to the no-record base (`HEAD~1...HEAD`, or `HEAD`), never
  to a later change's base, as the acceptance criterion of the full-commit-id change says.
- `is_canonical_commit_id` is `pub(crate)` and `change status` evidence uses it instead of a copy.
- It accepts SHA-1 ids only. A SHA-256 repository would see every recorded commit treated as absent; that is
  the existing recording format, and noted here for when SpecSync records SHA-256 ids.
- The order test's records carry full commit ids.
