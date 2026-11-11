---
change: after-approve-next-action-names-check-commit-and-review-and-ship-refuse-a-verification-commit-that-lacks-the-change-s
artifact: requirements
---

# Requirements

The proposed contract is in `deltas/change.md`. In brief:

- REQ-change-156 (added): after approve, status and show name
  `specsync change check <id> --commit` for an approved or implementing
  change, not a bare `change check`.
- REQ-change-157 (added): `change review` and `change ship` refuse when
  the verification commit does not contain the change's work, and do not
  bind a review to such a commit. Uncommitted `.specsync/` evidence alone
  does not trigger this.
- REQ-change-132 (modified): the open-task status line still names the
  task first; the check command that follows is `change check <id> --commit`.

Invariant 59 restates those two rules in `change.spec.md`, and is claimed
but not inserted until after definition approval. Bare `change check`
without `--commit` stays valid (REQ-cli-args-010).
