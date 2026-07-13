---
change: CHG-0016-reject-modified-definitions-when-reaccepting-an-already-applied-change
artifact: plan
---

# Plan

1. Compare reopened definition identity with the immutable pre-reopen contract before reacceptance.
2. Reject missing audit history and changed definitions before recording closing approval or writing files.
3. Preserve successful delivery-only reopen and no-double-application behavior.
4. Add unit and CLI regressions that approve and verify a modified reopened definition, then prove acceptance rejects it.
5. Update the canonical lifecycle contract, workflow guide, testing evidence, and changelog.
6. Run formatting, Clippy, unit and integration tests, strict specs, docs, audit, build, and full Trust before approvals.
