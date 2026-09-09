---
change: close-the-specsync-6-0-0-p1-release-defects-found-in-overnight-proving
artifact: plan
---

# Plan

1. Reconstruct the twelve worktree fixes onto `leif/specsync-6-p1-fixes` from `origin/main` at `0d0251bf`.
2. Pin each P1 with a unit or integration test.
3. Update README, site quickstart/cli/mcp-security, CHANGELOG.
4. One change package covering every touched module; approve as 0xLeif with the overnight-brief note.
5. `change check --commit`, `fledge lanes run pre-push`, push, wait for CI.
6. `change review --reviewer 0xLeif`, `change ship` with no commit in between, commit the archive tip, push.
7. Open the PR, comment `fixed in PR #N` on #656/#653/#768 without closing them, leave #769 unmerged.
8. Write `docs/6-0-confidence-report.md` and re-run `fledge lanes run verify` before calling the work done.
