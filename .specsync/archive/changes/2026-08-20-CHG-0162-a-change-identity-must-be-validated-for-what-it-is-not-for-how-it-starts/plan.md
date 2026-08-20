---
change: CHG-0162-a-change-identity-must-be-validated-for-what-it-is-not-for-how-it-starts
artifact: plan
---

# Plan

1. Measure the longest change ID in the archive before choosing a ceiling — 90 bytes, so a
   255-byte component limit leaves ample room and needs no per-shape exception.
2. `validate_change_id`: drop the `starts_with("CHG-")` test; add non-empty, a
   `MAX_CHANGE_ID_BYTES` ceiling, and the shared reserved-name predicate. Keep the single-
   component, separator and control-character checks unchanged.
3. Three tests: a slug-only ID is accepted, an unsafe or unbounded one is refused, and every
   historical shape stays legal as the control.
4. Discriminate against a separate checkout of `origin/main`.
