---
change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
artifact: tasks
---

# Tasks

- [x] Commit the failing starling test and the ship-status blocker test.
- [x] Implement the reference commit, id collection, accounting and refusal message.
- [x] Call it at check, acceptance, verify and audit; add the ship-status blocker.
- [x] Tests for accounted removals, merged-in removals, moves, renumbers, deleted and legacy files, the fallback, skip and unreadable cases, and audit.
- [x] Docs (`deltas.md`), CHANGELOG, spec invariant and lesson.
- [x] `change check --commit`, verify lane, `change audit --strict`, trust.
- [x] ss-lead's review of b8018cc0 and the tester's run: skip .git, .specsync and exclude_dirs; assert ship-status's exit 1 with #654 on main; skip an unreadable change; natural id order; name the way out of every unreadable Git read, unshallowing a shallow clone.
