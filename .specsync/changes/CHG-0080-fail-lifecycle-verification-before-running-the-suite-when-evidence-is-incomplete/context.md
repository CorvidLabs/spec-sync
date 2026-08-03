---
change: CHG-0080-fail-lifecycle-verification-before-running-the-suite-when-evidence-is-incomplete
artifact: context
---

# Context

Three defects surfaced by driving CHG-0079 (PR #499) through the lifecycle. Each cost a full
re-verification — roughly ten minutes — to discover something the workspace already described.

Evidence completeness was resolved after the verification commands ran, so a missing
`## Requirement evidence` row was reported only once the whole suite had executed. The message
then pointed at `verification.json`, which records the outcome but cannot contain the fix.

`change check` could not run twice. The first run applied an `## ADDED` block to the canonical
spec; the second rejected it as `cannot add existing block`, leaving the change unverifiable
without hand-editing the tree.

Independent worktrees allocate from their own sequence ledger, so two handed out CHG-0078 for
different work. Nothing detected it, because the ordinal only stops identifying a single change
once both land.
