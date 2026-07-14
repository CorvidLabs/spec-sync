---
change: CHG-0026-keep-lifecycle-recursion-detection-private-while-preserving-deterministic-nested
artifact: context
---

# Context

The recursion context marker exists only to stop a configured verification command from re-entering a SpecSync lifecycle command in a child process. It is not a reusable library contract. Keeping the marker and its diagnostic helper in the binary entry point lets every lifecycle dispatch share the guard without exporting internal process state from the `change` module.
