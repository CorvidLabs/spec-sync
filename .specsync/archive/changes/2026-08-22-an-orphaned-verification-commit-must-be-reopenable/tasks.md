---
change: an-orphaned-verification-commit-must-be-reopenable
artifact: tasks
---

# Tasks

- [x] Read the whole `reopen` path end to end rather than a range around the reported line
- [x] Find where `check` detects unreachability and determine whether it is reusable
- [x] Extract ONE anchor resolver instead of adding a third idiom
- [x] Search for parallel implementations of both the anchor disjunction and the digest-equality
      staleness test; classify every hit
- [x] Patch the sibling at `:1979` to read a recorded cause rather than infer from digest equality
- [x] Record the cause in `ReopenRecord` with `skip_serializing_if` so existing ledgers stay
      byte-identical
- [x] Prove discrimination against a SEPARATE CHECKOUT, not a revert in place
- [x] Add the vacuity control so the widening is bounded, not total
- [x] Confirm a tampered archive is still refused on the new path
- [x] Amend invariants 15 and 18, REQ-change-017/018/034/035, the Error Cases rows, and document
      the new `ReopenCauseV1` export
- [x] Full suite, clippy, fmt, `specsync check`
