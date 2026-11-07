---
change: print-a-single-choice-interview-answer-literally-in-the-approval-refusal-so-the-named-change-answer-command-runs-as
artifact: requirements
---

# Requirements

- REQ-change-133 (modified, `deltas/change.md`). A single-choice question names its answer itself (`constraints none`), so the command runs as printed, and `constraints` adds that the lessons or principles that apply can be named instead. A draft missing both its specs and its paths is refused for the specs first, the question status asks first. The guard refuses exactly what it refused before.
- REQ-cmd-change-answer-hint-001 (added, `deltas/cmd_change.md`). Text and JSON `change status` name, for a draft's open interview question, the same `change answer` command approve's refusal names.

Human intent: DOCS-3.a (approved 2026-11-05), and DOCS-3.b and DOCS-3.c, drafted for #958 and approved by Leif as drafted in the Herdr orc tab on 2026-11-06 at about 19:35 MST ("1A 2yes"), relayed by orc t18. They are captured in `hi/docs.md` in this change.
