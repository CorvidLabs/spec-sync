---
change: restore-cmd-scaffold-s-3md-lessons-move-the-second-z-5-plane-to-z-6
artifact: testing
---

# Testing

Bug-fix, no spec change. Observable before and after, using the branch build of main
`ad8544e9`:

- Red, at main with the duplicate: in a throwaway worktree, `specsync change new … --spec
  cmd_scaffold` briefs "Lessons: … specs/cmd_scaffold/context.md (13 line(s))". The 3md file
  did not parse, so the fallback was used.
- Green, with the second plane at `z=6`: the same command briefs "Lessons: …
  specs/cmd_scaffold/cmd_scaffold.3md (16 line(s))".
- Planes are now z=0 to z=6, unique and in order. `specsync check --strict --require-coverage
  100` passes, and `change check --commit` runs last. The full suite runs in hosted CI.

No new automated test: the general guard, which reports an unparseable 3md, is #1060's scope.
