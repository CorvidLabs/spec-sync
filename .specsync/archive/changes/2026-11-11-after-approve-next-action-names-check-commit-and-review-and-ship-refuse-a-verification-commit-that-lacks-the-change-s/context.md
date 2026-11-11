---
change: after-approve-next-action-names-check-commit-and-review-and-ship-refuse-a-verification-commit-that-lacks-the-change-s
artifact: context
---

# Context

Issue #535, as re-measured on 2026-08-14 and again on current main
(`ad8544e9`, specsync 6.2.0, 2026-11-11 MST).

The filing made two claims. The second — that the resulting archive cannot be
committed, with every recovery verb refusing — does not hold on 6.0.0 or on
6.2.0: `git add` and `git commit` succeed, and there is no hook refusal. That
half is out of scope here.

The first claim still holds. After `change approve`, `change status` and
`change show` name `specsync change check <id>` with no `--commit`. Following
that command, then `change review` and `change ship`, all succeed against a
tree where the change's work is uncommitted. Review binds to the seed commit,
which does not contain the work. On 6.2.0, `change ship` now writes the
archive commit itself and includes the uncommitted work in that commit, while
`verification.json` and `review.json` still name the seed. `ship-status` then
reports the review stale. The guided path is still not sufficient, and
review/ship still accept a verification commit that lacks the work.

After a bare `change check`, text `next_action` already says
`change check <id> --commit` if needed. The gap is after approve (and the
same next step in `implementing`).

`change ship-status` already advertises `change check <id> --commit` for an
approved change (REQ-cmd-change-033). Handoff already calls uncommitted
edits under `affected_paths` `conditional` (REQ-change-093, invariant 42).
Neither stops review or ship, and neither is the `Next:` line after approve.

This is READY: it adds and changes requirement text. CHANGE-7 already says
the tool must name the single next thing to do; the criteria below come from
the issue and the later re-measurement, not from new human-intent wording.

Out of scope, deliberately: making `--commit` the default for `change check`
(that would change REQ-cli-args-010; bare check stays valid for a first
pass), and restoring the refuted unrecoverable-hook refusal.
