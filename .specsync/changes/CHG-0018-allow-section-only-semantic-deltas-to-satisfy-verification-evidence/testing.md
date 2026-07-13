---
change: CHG-0018-allow-section-only-semantic-deltas-to-satisfy-verification-evidence
artifact: testing
---

# Testing

`REQ-change-019` is covered by `change::tests::section_only_semantic_delta_can_satisfy_acceptance_evidence` and `change::tests::missing_semantic_acceptance_evidence_is_not_reported_as_command_failure`.

- Verify a modified spec-section-only delta passes with non-empty acceptance criteria and no requirement IDs.
- Verify a removed-only semantic delta fails with the semantic-evidence diagnostic rather than a command-failure diagnostic.
- Retain requirement evidence collection and missing-mapping checks unchanged.
