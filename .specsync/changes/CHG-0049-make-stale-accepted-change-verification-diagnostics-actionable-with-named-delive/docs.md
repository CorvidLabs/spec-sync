---
change: CHG-0049-make-stale-accepted-change-verification-diagnostics-actionable-with-named-delive
artifact: docs
---

# Docs

The `change check` stale-evidence error is self-explanatory after this change: it names the
delivery input whose bytes moved, its canonical owner module, and the recovery path. When no
accepted or archived successor covers the input, the message tells the operator to run
`specsync change reopen <id>`; when a covering successor exists but is itself stale, the message
names those successor changes so the operator verifies and accepts them first. No workflow doc
page changes are required; the canonical `change` spec Error Cases table records the new
diagnostic behavior.
