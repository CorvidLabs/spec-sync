---
change: watch-outcome-tests-wait-for-the-verdict-line
artifact: context
---

# Context

#1040: fixed 5s timeout flakes under load (same class as #707). Test-only: read stdout until the verdict line appears (120s bound for a hung child), then kill. t370 assigned this as a bug fix; no requirement text added.
