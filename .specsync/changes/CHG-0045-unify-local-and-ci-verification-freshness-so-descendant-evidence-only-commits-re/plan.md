---
change: CHG-0045-unify-local-and-ci-verification-freshness-so-descendant-evidence-only-commits-re
artifact: plan
---

# Plan

1. Extract one environment-independent freshness predicate that validates ancestry, approved contract digest, project-input digest, persisted state/evidence consistency, and every intervening commit edge.
2. Replace the local/CI branch in strict lifecycle checking and the exact-HEAD-only summary closure with the shared predicate.
3. Parse NUL-delimited paths for every intervening commit against every parent and accept only the exact supported verification-persistence filenames below canonical active-change IDs.
4. Add focused unit regressions for one and multiple supported persistence commits, source-change-then-revert, disallowed lifecycle paths, malicious state mutation, mixed commits, merge handling, nonancestor history, and local/CI parity.
5. Add CLI integration coverage proving `change status` and strict `change check` agree after persisted evidence commits.
6. Update `REQ-change-013`, `REQ-change-016`, and their canonical testing/mapping companions without weakening exact closing-approval rules.
7. Run focused tests, the complete native suite, format, type, lint, build, strict SpecSync, and Trust verification.
8. Reapprove and reverify CHG42 through CHG44 only after CHG45's implementation and native gates are complete; do not record closing approval, acceptance, release, or hosted success locally.
