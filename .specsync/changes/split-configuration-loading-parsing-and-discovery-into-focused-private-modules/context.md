---
change: split-configuration-loading-parsing-and-discovery-into-focused-private-modules
artifact: context
---

# Context

The owner requested smaller spec-sync files, continuation, and a branch with a commit. The delegated scope is configuration only on main 815690d. src/config.rs has 3365 lines, including 98 tests. Existing APIs and security-sensitive retained parsing are fixed constraints. Source bodies and regressions will move intact; no requirement behavior changes.

The definition approval records the owner's existing authorization, entered by corvid-agent. Independent agent review is additional engineering evidence and does not stand in for the later human implementation review or authorize shipping.
