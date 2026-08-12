---
change: CHG-0105-make-drift-gate-by-default-flip-the-default-enforcement-mode-from-warn-to-stric
artifact: docs
---

# Docs

`CHANGELOG.md` records the default change and names the opt-out (`--enforcement warn`), since
this alters exit codes for every existing consumer of a bare `specsync check`.
