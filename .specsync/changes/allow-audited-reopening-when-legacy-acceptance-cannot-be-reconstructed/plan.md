---
change: allow-audited-reopening-when-legacy-acceptance-cannot-be-reconstructed
artifact: plan
---

# Plan

Reproduce #751 with a real Git acceptance transition whose tree differs from the signed raw inputs while the current tree matches. Use the archive reconstruction predicate during legacy reopen eligibility. Extend the audit cause and sequence-history validation. Run the regression before and after the fix, then existing reopen and reconstruction tests and repository verification.
