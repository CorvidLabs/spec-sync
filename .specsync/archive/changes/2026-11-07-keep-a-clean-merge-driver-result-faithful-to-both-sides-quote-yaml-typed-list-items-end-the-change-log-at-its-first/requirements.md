---
change: keep-a-clean-merge-driver-result-faithful-to-both-sides-quote-yaml-typed-list-items-end-the-change-log-at-its-first
artifact: requirements
---

# Requirements

- REQ-merge-006 (added): the git merge driver reports a merge clean only when the result keeps what both sides wrote and meant; otherwise it leaves a conflict. It covers the four shapes in #872, #875, #876 and #870.
- Traces to HI MERGE-5, MERGE-5.a (#872), MERGE-5.b (#875), MERGE-5.c (#876) and MERGE-5.d (#870), approved by Leif on 2026-11-06 and relayed by the orc. Also MERGE-1 (either order) and MERGE-4 (no looser gates).
- REQ-merge-006 is assigned by the orc's registry.
