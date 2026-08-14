---
change: CHG-0126-the-config-refusal-must-guard-both-loaders-because-load-config-is-a-second-door
artifact: tasks
---

# Tasks

1. Split `load_config` into a refusing default and a named permissive variant.
2. Point `wizard` and the registry initialiser at the permissive one.
3. Verify every direct caller either refuses or is a deliberate repair path.
4. CHANGELOG entry.
