---
change: change-audit-reports-a-delta-item-that-targets-the-frozen-change-log
artifact: context
---

# Context

Issue #1090, filed from fable-lead's note on #1088's review. With #1088, `change approve`, `change check`, verification and acceptance refuse a delta item that targets the frozen `## Change Log` (REQ-change-159), but the project audit (`check_project_with_command_output`, behind `change audit`) never asks that refusal, so `change audit --strict` passes an approved change that `change check` then refuses. Leif approved the definition (ss-lead/gates/1090-definition.md) "Approve as written" on 2026-11-19 at about 14:19Z, relayed by hc t4c through ss-lead hc t58; it was to be built once #1088 merged (ee76ef1b on main).
