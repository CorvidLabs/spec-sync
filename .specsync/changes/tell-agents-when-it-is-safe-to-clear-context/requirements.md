---
change: tell-agents-when-it-is-safe-to-clear-context
artifact: requirements
---

# Requirements

New:

- `REQ-change-093` — the lifecycle SHALL compute a handoff readiness (`safe`, `conditional`,
  `not-yet`) for every change from its lifecycle signals, with a plain-language reason, the resume
  command, and the steps to take before clearing when it is not safe.

Modified:

- `REQ-cmd-change-005` — `status`, `show`, a passing `check`, `approve`, `review`, `finalize`, and
  ship's finalize end with the `Handoff:` line in text; JSON carries `summary.handoff` (and `handoff`
  on the approve transition).
- `REQ-agents-check-audit-commands-001` — skill prose tells agents to clear context only when the
  `Handoff:` line says `safe`, and otherwise to do what `before clearing` names first.
