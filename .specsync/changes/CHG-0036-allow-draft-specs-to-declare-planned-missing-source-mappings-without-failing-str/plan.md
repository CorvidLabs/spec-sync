---
change: CHG-0036-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str
artifact: plan
---

# Plan

1. Add the default-false draft-file enforcement setting to shared configuration and both readers/writers.
2. Represent non-failing planned mappings as first-class validation notices.
3. Preserve path safety and real-file validation while exempting only safe missing draft paths.
4. Validate duplicate ownership for existing mapped files across draft and non-draft specs.
5. Keep coverage based only on real discovered source files and expose notices in every check format.
6. Add transition, mixed-status, configuration, safety, ownership, and exact denominator regressions.
7. Apply canonical deltas, run the full Fledge lane and strict 100 percent validation, then require exact-head hosted checks.
