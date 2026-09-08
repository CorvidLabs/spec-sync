---
change: repair-executable-lifecycle-demos
artifact: context
---

# Context

All three advertised runnable examples fail with the preserved SpecSync 6 RC16 binary because they construct retired CHG-number IDs. The five-epic script also reports six passing product tests without invoking cargo test. These defects prevent using the examples as release evidence. This is a separate scope from promotion hardening; no runtime behavior or release protection changes.
