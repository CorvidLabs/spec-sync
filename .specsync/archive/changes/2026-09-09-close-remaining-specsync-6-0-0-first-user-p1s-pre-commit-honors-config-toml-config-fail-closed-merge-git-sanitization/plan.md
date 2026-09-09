---
change: close-remaining-specsync-6-0-0-first-user-p1s-pre-commit-honors-config-toml-config-fail-closed-merge-git-sanitization
artifact: plan
---

# Plan

1. Branch `grok/specsync-6-remaining-p1s` from `origin/main` at `7df304a` (includes #773; leave the Action pin).
2. One change package owning hooks, config, merge, git_utils, change.
3. Implement the three P1s with fail-then-pass tests; v1 next_action; `--kind bug-fix`; MIGRATION.md; confidence-report rows; CHANGELOG.
4. Approve as 0xLeif with the overnight-brief note.
5. `change check --commit`, `fledge lanes run pre-push`, push, wait for CI.
6. `change review --reviewer 0xLeif`, `change ship` with no commit in between, commit the archive tip, push.
7. Open one PR, leave it open, comment `fixed in PR #N` on addressed issues without closing them.
