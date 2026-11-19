---
change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
artifact: tasks
---

# Tasks

- [ ] Commit the failing starling test and the ship-status blocker test.
- [ ] Implement the reference commit, id collection, accounting and refusal message.
- [ ] Call it at check, acceptance, verify and audit; add the ship-status blocker.
- [ ] Tests for accounted removals, merged-in removals, moves, renumbers, deleted and legacy files, the fallback, skip and unreadable cases, and audit.
- [ ] Docs (`deltas.md`), CHANGELOG, spec invariant and lesson.
- [ ] `change check --commit`, verify lane, `change audit --strict`, trust.
