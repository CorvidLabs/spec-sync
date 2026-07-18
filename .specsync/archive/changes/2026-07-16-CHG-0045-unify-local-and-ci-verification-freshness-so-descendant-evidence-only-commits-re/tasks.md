---
change: CHG-0045-unify-local-and-ci-verification-freshness-so-descendant-evidence-only-commits-re
artifact: tasks
---

# Tasks

- [x] Reproduce and isolate the local-versus-CI freshness mismatch on truthful committed evidence.
- [x] Identify the strict-check and `summarize_change` decision paths.
- [x] Define the shared ancestry, digest, state/evidence-consistency, and per-parent intervening-commit trust boundary.
- [x] Define the canonical `REQ-change-013` and `REQ-change-016` updates and regression matrix.
- [x] Implement fail-closed NUL-delimited enumeration of every intervening commit and parent edge.
- [x] Restrict descendants to the three supported verification-persistence filenames below canonical active-change IDs.
- [x] Route strict checks and summaries through the shared predicate.
- [x] Add one-child, multiple-child, change-then-revert, disallowed-path, malicious-state, mixed-commit, merge, nonancestor, and environment-parity regressions.
- [x] Update canonical change specification companions.
- [x] Run and record focused and complete native verification.
- [x] Prepare lifecycle-aware SpecSync and Trust verification without claiming hosted or closing results early.
