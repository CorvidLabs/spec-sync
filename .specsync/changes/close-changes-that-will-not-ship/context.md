---
change: close-changes-that-will-not-ship
artifact: context
---

# Context

A change that will never ship had no exit: deleting its workspace by hand left the sequence
ledger, archive CI and `ship-status` counting it, and a verifying change replaced by a new one
could not be handed over (#1070 item 1, #844 item 2, #541; 12 stale Jackdaw changes).

Leif chose to build "change close command" at 2026-11-13T03:28Z and answered the design questions
at 03:52Z (coordinator chat, Claude Code hc t370, relayed by claude-code-t370):

- Q1, where a closed change goes: "Into the archive, marked closed (Recommended)".
- Q2, closing many: "Several ids in one command (Recommended)".
- Q4, replacing a verifying change: "close --replaced-by (Recommended)".
