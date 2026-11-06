---
change: a-cold-archived-change-status-answers-every-archive-s-workflow-version-history-from-one-history-walk-and-the-command-s
artifact: requirements
---

# Requirements

- REQ-change-140: inside a change read scope, every archived change's workflow-version history comes from one lifecycle-state history walk per command, with the same verdicts, refusals and bounds as the per-archive check, and a fallback to it when the walk cannot answer.
- REQ-change-141: the state-file reads and path-exists checks behind that validation use the command's one object batch reader on the command's thread; an unreadable listed blob still refuses and is never cached as absent.

Traces SPEED-1.a, SPEED-1.a.1 and SPEED-1.d.
