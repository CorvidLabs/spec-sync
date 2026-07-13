---
change: CHG-0021-close-reopened-lifecycle-review-gaps
artifact: testing
---

# Testing

`REQ-change-020` is covered by
`change::tests::reaccept_rejects_definition_changes_after_canonical_application`,
`change::tests::accepted_evidence_survives_squash_merge_from_nested_project_root`,
and `change::tests::reopen_rejects_current_evidence_and_requires_explicit_audit_fields`.

The regressions prove check/accept parity, state retention, current-input
rejection after an independent history failure, and top-relative history
detection from a nested project root. Full validation uses the Fledge
repository lane and strict 100% SpecSync coverage.
