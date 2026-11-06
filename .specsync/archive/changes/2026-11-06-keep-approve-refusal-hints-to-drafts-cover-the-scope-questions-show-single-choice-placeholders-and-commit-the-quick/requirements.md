---
change: keep-approve-refusal-hints-to-drafts-cover-the-scope-questions-show-single-choice-placeholders-and-commit-the-quick
artifact: requirements
---

# Requirements

- REQ-change-133 (modified, `deltas/change.md`). Approve's refusals for an unfinished draft now also cover a missing affected path or spec. The command is named only for drafts. A question's choices, including a single one, are its placeholder. The guard refuses exactly what it refused before.

Human intent: DOCS-1, DOCS-3 and DOCS-3.a. Leif approved them on 2026-11-05 at 20:22 MST, relayed by orc t18. This change makes the implementation meet them and REQ-change-133. No new criteria.
