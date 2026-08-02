---
change: CHG-0077-reuse-successful-exact-pr-ci-provenance-across-metadata-only-descendants-and-pre
artifact: plan
---

# Plan

1. Port the bounded first-parent check-reuse helper and focused fixtures from draft PR #490.
2. Use the helper in Trust and archive finalization for review/archive metadata descendants.
3. Make trusted-policy selection prefer an authenticated success for the exact SHA over later
   cancelled or failed republications or reruns, binding the successful publication to its immutable
   workflow run attempt.
4. Preserve every existing GitHub App, PR, SHA, workflow, repository, and bounded-history check.
5. Run the focused Python/workflow regressions, one independent scoped review, and one final
   repository verification before same-PR finalization.
6. Dogfood a product tip followed immediately by review/finalization metadata in the private sandbox.
