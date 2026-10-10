---
change: refuse-a-fenced-plan-and-keep-a-damaged-change-directory-open
artifact: context
---

# Context

A fenced example in a plan was being read as the plan. A change directory that still held files but had lost state.json was being treated as finished. Both are closed. An empty directory husk is still skipped.
