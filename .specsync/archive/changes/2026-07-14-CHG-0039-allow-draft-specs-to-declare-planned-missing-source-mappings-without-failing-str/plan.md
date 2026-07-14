---
change: CHG-0039-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str
artifact: plan
---

# Plan

1. Add the default-false draft-file enforcement setting to shared configuration and both readers/writers.
2. Represent non-failing planned mappings as first-class validation notices.
3. Preserve path safety and normalize or reject redundant segments before exempting safe missing draft paths.
4. Build ownership from every discovered spec while validating only the requested incremental subset.
5. Keep coverage based only on real discovered source files and expose notices in every check format.
6. Update all six canonical contracts affected by fields, signatures, rendering, and validation behavior.
7. Add transition, mixed-status, configuration, safety, ownership, incremental-cache, normalization, and exact denominator regressions.
8. Apply canonical deltas, run the full Fledge lane and strict 100 percent validation, then require exact-head hosted checks.
