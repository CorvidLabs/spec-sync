---
change: CHG-0054-trust-accepted-change-evidence-that-is-recorded-in-main-history-by-squash-merged
artifact: docs
---

# Docs

Document that squash-merged pull requests are fully supported: accepted-change archival trusts any
in-history commit that records the change as accepted with byte-identical evidence, so discarding
the original acceptance-transition commit in a squash merge no longer blocks archival.
