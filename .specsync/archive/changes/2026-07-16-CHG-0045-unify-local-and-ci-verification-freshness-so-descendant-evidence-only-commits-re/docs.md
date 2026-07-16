---
change: CHG-0045-unify-local-and-ci-verification-freshness-so-descendant-evidence-only-commits-re
artifact: docs
---

# Docs

Update the canonical change lifecycle documentation rather than adding release marketing. `specs/change/requirements.md` will clarify that local and hosted strict checks use identical verification-freshness semantics and that only the three supported persistence filenames below a canonical active-change ID may appear on intervening parent edges. `specs/change/testing.md` will identify per-commit/per-parent inspection, source-change-then-revert, disallowed lifecycle paths, malicious state mutation, mixed commits, merges, and divergent history as mandatory regression boundaries. `specs/change/change.spec.md` will map the shared predicate and regression tests to `REQ-change-013` and `REQ-change-016`.

Public documentation must not suggest setting `CI=true`, manually editing verification JSON, broadening the path allowlist to all volatile inputs, or recording closing approval early. The supported workflow remains `specsync change verify`, commit generated evidence, strict check, hosted check, then closing approval only on the exact authorized head.
