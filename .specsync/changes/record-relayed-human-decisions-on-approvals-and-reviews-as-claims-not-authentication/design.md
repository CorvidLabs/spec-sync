---
change: record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication
artifact: design
---

# Design

- `RelayedDecision { decided_by, words, where, recorded_by }` (all `String`). It deserializes through a validating `TryFrom<serde_json::Value>`: the value must be an object with exactly those four string keys, each non-blank and at most 4096 bytes. The field on both records is `#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = ...)]`, so absent reads as `None` and `null` is refused.
- `relayed_decision(rerun, decided_by, words, where_, recorded_by)` builds the record from flags. With none of the three flags it returns `None`; with some, it refuses and names each missing flag plus the full re-run. `decided_by` comes from `--actor` or the validated `--reviewer`.
- `approve_definition_with_relay` passes the relay into `append_approval` and the portable pair, and refuses a relay whose `decided_by` is not the explicit actor. The old `approve_definition*` functions call it with `None`.
- `record_scoped_review_with_relay` sets `relayed_by` on the new review, and the attempt ledger keeps the same record. `scoped_review_digest(&review)` hashes the record with `relayed_by` cleared; finalization creation, finalization validation and the collapsed-archive check all use it.
- `load_approvals` and `load_scoped_review` add `git checkout <commit> -- <path>` to a refusal caused by a malformed relay record.
- `recorded_claims(root, record)` reads `approvals.json` and `review.json` wherever the change lives and returns digest-free `RecordedClaim`s. Text status prints `Approval: <actor> (a recorded claim, not authenticated)` and `Review: <reviewer> (<verdict>; a recorded claim, not authenticated)`, each followed by a `Relayed:` line when there is a relay. JSON adds `recorded_claims`, omitted when empty.
