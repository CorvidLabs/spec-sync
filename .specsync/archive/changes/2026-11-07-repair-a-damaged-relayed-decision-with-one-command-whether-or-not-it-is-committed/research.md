---
change: repair-a-damaged-relayed-decision-with-one-command-whether-or-not-it-is-committed
artifact: research
---

# Research

- `with_relayed_read_recovery(root, path, message)` builds every damaged-relay refusal; `load_approvals` and `load_scoped_review` (approvals, review, every attempt) call it. `recorded_claims` reads `approvals.json` and `review.json`, so `change status` refuses those. Status swallows `load_scoped_review` errors, so a damaged attempt surfaces through `change review` or `change finalize`.
- `review.json` must equal the last entry of `review-attempts.json`; the reader refuses a mismatch. So the two are copies of each other for the latest review.
- No digest reads a relay: the review digest, both correction digests and the review-history comparisons clear or ignore it ("Keep it out"), the approval digest never hashed the actor, and `.specsync/changes/` is volatile in the project input digest. Repairing a relay therefore stales nothing.
- serde_json is built without `preserve_order`, so a write that should reproduce SpecSync's bytes must round-trip through the typed ledger (`ApprovalLedger`, `ScopedReviewAttemptLedger`, `ScopedReviewRecord`) and `json_content`.
- `git_file_at_commit(root, "HEAD", path)` reads a committed copy, returning `None` when the file is not at `HEAD`.
- Finalize commits the archive it writes (`chore(lifecycle): archive <id>`), so an archived package is always at `HEAD`.
