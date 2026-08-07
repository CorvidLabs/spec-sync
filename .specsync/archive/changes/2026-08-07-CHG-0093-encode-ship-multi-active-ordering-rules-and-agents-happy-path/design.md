---
change: CHG-0093-encode-ship-multi-active-ordering-rules-and-agents-happy-path
artifact: design
---

# Design

Local-only guidance (no GH API). Warnings are additive; they do not hard-block ship when ready_to_finalize, because sequential finalize is required and ship must remain usable for the first change in a multi-active set.
