## MODIFIED

### REQUIREMENT REQ-cmd-change-007

`specsync change ship-status` SHALL report ship readiness for an active change,
including local tip classification and optional live GitHub check-run trust:

- HEAD tip class: `product`, `review_only`, `archive_only`, or `other`
- verification tip presence and ancestry relative to HEAD
- whether a scoped review is recorded
- trust guidance for staged product → review → archive tips
- when `GITHUB_TOKEN` is set and the git remote is GitHub, live check-run trust for the
  parent commit SHA (falling back to HEAD) with `trust.status` in
  `green` | `pending` | `failed` | `empty` | `unavailable` and
  `trust.source` = `github_check_runs`
- when the token is absent, lookup fails, or `SPECSYNC_SHIP_LOCAL_GUIDANCE` is set,
  `trust.status` is `local_guidance` or `unavailable` and the command still succeeds
- ordered ship stages with concrete next actions
- blockers, warnings (including merge-before-finalize), and `ship_next`
- sibling active change ids and multi-active ordering warnings when present

Acceptance Criteria

- JSON includes `tip_class`, `tip_sha`, `parent_sha`, `trust`, `stages`,
  `verification_present`, `verification_ancestor_of_head`, `review_present`,
  `ready_to_finalize`, `blockers`, `warnings`, `sibling_active_ids`, and `ship_next`.
- Tip class is derived from the paths changed in HEAD relative to its first parent
  (or working-tree vs HEAD when HEAD is not a useful tip).
- An absent or non-ancestor verification commit is a blocker naming re-check.
- A non-archived change always warns not to merge while the change is still active.
- When other active changes exist, warnings name finalize-one-at-a-time, do-not-batch-reviews,
  and do-not-merge-with-active-changes rules.
- Live trust queries never fail the ship-status command; errors appear under `trust.error`
  with `trust.status` = `unavailable`.
- Offline and no-token runs keep `trust.status` = `local_guidance` without network access.
