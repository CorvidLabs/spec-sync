---
change: repair-executable-lifecycle-demos
artifact: testing
---

# Testing

Baseline: all three original scripts exit 1 at the retired change ID lookup. After approval, run all repaired examples with the current SpecSync 6 binary and assert their lifecycle finishes with no active changes. Execute independent negative controls restoring a retired ID and forcing product-test failure. The five-epic demo must run its generated product tests; configured verification commands are not test evidence. Run the Python harness in CI against the actual built binary. Run full local verify, strict specs, pre-push, Trust, and signed provenance policy before completion. CI and human implementation review remain separate gates.
