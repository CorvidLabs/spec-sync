---
change: CHG-0034-support-extensionless-source-discovery-through-an-explicit-include-extensionless
artifact: plan
---

# Plan

1. Add and round-trip the additive configuration field with default-preservation tests.
2. Centralize extensionless-aware source selection and update every configuration-driven scanner.
3. Add unit and CLI integration regressions for extensionless-only and mixed projects, asserting non-zero strict file and LOC coverage.
4. Make wizard discovery testable and prove matching directories are excluded while regular extensionless files remain eligible.
5. Update canonical config and validator specs plus public configuration documentation.
6. Run formatting, focused tests, strict SpecSync coverage, and the complete Fledge verification lane.
