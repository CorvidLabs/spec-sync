---
change: CHG-0033-close-final-5-0-2-lifecycle-review-and-intent-preservation-gaps
artifact: plan
---

# Plan

1. Approve this exact definition and semantic delta.
2. Add failing focused tests for manifest selection, canonical companion scope, and prose parsing.
3. Implement the smallest question-aware and path-aware fixes in `src/change.rs`.
4. Update the change module spec and companions to match the implementation.
5. Run focused and full repository verification, then the Trust and Attest gates.
6. Obtain closing approval, accept CHG-0033, resolve the review threads, and update PR #370.
