---
change: CHG-0017-allow-audited-reopen-after-squash-and-canonical-successors
artifact: testing
---

# Testing

- Create and accept a change on a feature branch, squash it to main, and record the remote base.
- Accept a later change governing the same source on top of the squash.
- Prove the original verification commit is unreachable and remote-workspace equality is false.
- Prove recorded acceptance and successor governance are present, then reopen with an explicit audit reason.
- Retain rejection tests for evidence that is neither integrated nor recorded in current history.

`REQ-change-018` is covered by `change::tests::squash_merged_acceptance_reopens_after_a_current_canonical_successor`, `change::tests::squash_fallback_rejects_unintegrated_or_changed_evidence`, and `change::tests::reopen_rejects_current_evidence_and_requires_explicit_audit_fields`.
