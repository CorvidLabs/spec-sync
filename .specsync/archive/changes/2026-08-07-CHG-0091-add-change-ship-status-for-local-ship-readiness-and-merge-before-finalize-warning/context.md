---
change: CHG-0091-add-change-ship-status-for-local-ship-readiness-and-merge-before-finalize-warning
artifact: context
---

# context

`change ship-status` reports local ship readiness for one or all active changes:
verification commit presence and ancestry, scoped review presence, blockers, and
warnings including the merge-before-finalize trap. JSON includes the same fields.
Verifying `next_action` from status/show also names re-check and finalize-before-merge.

Evidence: CLI help exposes the subcommand; unit/manual run against a verifying change
with orphaned verification shows a blocker; clean verifying+reviewed reports ready_to_finalize.
