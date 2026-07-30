---
change: CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara
artifact: tasks
---

# Tasks

- [x] Implement one-approval state and historical two-approval compatibility
- [x] Implement targeted bidirectional implementation verification
- [x] Implement transactional same-PR `change finalize` and one archive layout
- [x] Implement safe lightweight archive-only CI and parent-check inheritance
- [x] Implement one scoped agent review and status guidance
- [x] Separate stable scope approval from volatile execution/evidence binding
- [x] Repair strict coverage and add bounded lifecycle snapshots
- [x] Port selected schema, ignore, exports, hooks, and agents fixes
- [x] Update canonical specs, docs, version surfaces, and changelog
- [x] Run focused regression suites

Independent scoped review, final release validation, and sandbox verification are
first-class lifecycle evidence gates. They are tracked by `change status` and
`testing.md`, not as implementation tasks that would circularly block
`change check`.
