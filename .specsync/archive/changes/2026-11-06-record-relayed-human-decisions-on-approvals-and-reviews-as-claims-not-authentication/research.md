---
change: record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication
artifact: research
---

# Research

- `definition_digest` and the scope digest never include `actor`; `ApprovalRecord` is compared field by field and never hashed as bytes, so a new optional field changes no digest.
- `ScopedReviewRecord` is hashed by `serde_json::to_vec` into `finalization.review_digest` (creation in the finalization gate, validation in `load_finalization`, and the collapsed-archive check). An older binary drops unknown keys before hashing.
- `resolve_actor` falls back to `git config user.name` and `$USER`, which would name the agent, not the person. A relay therefore requires an explicit `--actor`.
- `validate_scoped_reviewer_claim` trims the reviewer and limits it to 128 stable ASCII characters, so `decided_by` for a review is the trimmed claim.
- Text status deliberately avoids digest-bearing loaders (CodeQL alert #58); `RecordedClaim` holds names and relay strings only.
