---
change: CHG-0026-keep-lifecycle-recursion-detection-private-while-preserving-deterministic-nested
artifact: plan
---

# Plan

1. Move the verification context constant and diagnostic helper to the binary crate root.
2. Route change-module recursion checks through the private crate-root helper.
3. Remove the internal helper from the canonical exported-function table.
4. Run focused recursion tests, formatting and lint gates, then the configured full verification command.
