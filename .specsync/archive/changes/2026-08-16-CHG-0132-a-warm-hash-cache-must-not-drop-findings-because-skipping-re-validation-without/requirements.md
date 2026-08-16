---
change: CHG-0132-a-warm-hash-cache-must-not-drop-findings-because-skipping-re-validation-without
artifact: requirements
---

# Requirements

`REQ-hash-cache-00N` — a cached spec's validation result SHALL be stored and
replayed, so a warm run reports what a cold run reported.

`REQ-cmd-check-00N` — `check` SHALL produce identical findings on repeated runs
over an unchanged tree, in every format.

Out of scope: changing what the cache skips.
