---
change: synchronize-change-summary-state-after-verification
artifact: context
---

# Context

Issue #1143 reports that materialization writes implementing to state.json and change.md, then verification updates only state.json. The completed attempt already uses journaled publication, and the Markdown summary is outside the verification input digest. The workflow also directs people to put handoff notes in change.md.
