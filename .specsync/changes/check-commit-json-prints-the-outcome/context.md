---
change: check-commit-json-prints-the-outcome
artifact: context
---

# Context

Issue #839, re-cut per t370/t384: the behavior is required by REQ-change-004
and cli invariant 5, but REQ-cmd-change-unrun-commands-001's third acceptance
line pinned the empty-output behavior ("gains no success document"), so the
spec was self-inconsistent. t384's option (a): the outcome document carries
`configured_commands_not_run` like every other JSON success output, and the
AC/item-13 exemption is retired to match. READY track (retiring contract text
is Leif's call at merge).
