---
change: a-module-that-owns-a-changed-file-may-be-declared-without-a-delta
artifact: context
---

# Context

Every affected spec had to carry a delta ("semantic delta modules must exactly match affected
specs"). After a spec split moved a requirement into module B while module A still owns the code,
a change editing that code and B's requirement could not declare A, yet without A no declared
module owned its files (#844 item 4, #1070).

Leif's choice, 2026-11-13T03:52Z (coordinator chat, Claude Code hc t370, relayed by
claude-code-t370): for code owned by A with the requirement now in B he chose "Allow a code-only
module (Recommended)": a declared module with no delta is allowed when it owns one of the
change's files.
