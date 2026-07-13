---
change: CHG-0023-allow-squash-accepted-evidence-on-descendant-branches
artifact: testing
---

# Testing

- Extend `accepted_evidence_survives_integrated_squash_merge_and_archives` to create an empty descendant feature commit after the squash merge and assert closing evidence remains valid.
- Preserve the existing negative regressions for arbitrary off-history evidence, stale inputs, stale definitions, and mismatched approvals.
- Run all unit and integration tests because closing validation is shared by checks, status, acceptance, and archive.
- Run strict SpecSync validation at 100% file and LOC coverage and the full hosted matrix.
