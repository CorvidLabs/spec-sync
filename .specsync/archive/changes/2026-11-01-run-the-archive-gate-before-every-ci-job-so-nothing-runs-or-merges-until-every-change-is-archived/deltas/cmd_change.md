## MODIFIED

### REQUIREMENT REQ-cmd-change-007

`specsync change ship-status` SHALL report ship readiness for an active change,
including local tip classification and optional live GitHub check-run trust:

- HEAD tip class: `product`, `review_only`, `archive_only`, or `other`
- verification tip presence and ancestry relative to HEAD
- whether a scoped review is recorded, and whether the recorded review is still current
- archive-first trust guidance: verify and review locally, ship, then push the archive tip;
  CI runs the full lane only on a tip whose changes are all archived
- when `GITHUB_TOKEN` is set and the git remote is GitHub, live check-run trust for the
  HEAD commit SHA (falling back to the parent) with `trust.status` in
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
  `review_currency`, `ready_to_finalize`, `blockers`, `warnings`, `sibling_active_ids`,
  and `ship_next`.
- `review_currency` is `missing`, `unreadable`, `current`, `stale`, or `unavailable`, and
  `ready_to_finalize` is true only when it is `current`. Existence of `review.json` is not
  currency: finalization additionally requires the recorded review to still match the tree, so
  readiness that asked only whether the file existed recommended the very verb that then refused.
- A recorded review that is decidably out of date is a blocker naming what moved and the
  re-review that repairs it, and the review stage reports `current` rather than `done` so the
  named next action is the recovery instead of the refused verb.
- A recorded review whose currency could not be determined is reported as `unavailable` and is
  never reported as satisfied. It produces a warning stating that currency could not be
  determined and naming the re-review that re-anchors it, rather than a blocker, because whether
  an unobtainable guarantee ought to block is not a question this command settles.
- `ship-status` and `finalize` reach the same conclusion about the same change on the same tree:
  ship-status never reports a change ready that finalization will refuse on review currency.
- Tip class is derived from the paths changed in HEAD relative to its first parent
  (or working-tree vs HEAD when HEAD is not a useful tip).
- An absent or non-ancestor verification commit is a blocker naming re-check.
- A non-archived change always warns not to merge while the change is still active.
- When other active changes exist, warnings name finalize-one-at-a-time, do-not-batch-reviews,
  and do-not-merge-with-active-changes rules.
- Live trust queries never fail the ship-status command; errors appear under `trust.error`
  with `trust.status` = `unavailable`.
- Offline and no-token runs keep `trust.status` = `local_guidance` without network access.
- No trust guidance or stage action tells the operator to push a product tip and wait for
  remote CI before review, or to push a review/archive tip that reuses a parent's trust. A
  failed Archive check on a tip with an open change is described as expected, with ship and an
  archive-tip push as the next step.
- Stage ids stay `product_tip`, `review_tip`, `archive_tip` and `merge`; only their titles and
  actions describe the archive-first flow.

### REQUIREMENT REQ-cmd-change-019

The command SHALL make a pull request show whether its change is still open or already archived. An open change SHALL leave the pull request unfinished. When every change is archived, the pull request SHALL say it is ready to merge once the other checks are green. When it is time to archive, `change ship` SHALL archive the change, commit that archive, and say that merging is the step that is left. When it is not time, the same place SHALL say the one reason and the one command. When more than one change is open, the pull request SHALL say which one to archive first.

Acceptance Criteria
- The Archive check fails while a change workspace is open, names the one to archive first, and names one command.
- The Archive check passes only when no change workspace is open, and its message says the pull request is ready to merge when the other checks are green.
- The required CI gate fails while the Archive check fails.
- The Archive check runs before every other CI job. While it fails no other job runs, and both SpecSync implementation ready and the required CI gate fail.
- `change ship` on a ready change commits the archive tip and its next line says to push, wait until the checks are green, and then merge.
- `change ship` on a change that is not ready names one reason and one command and does not say to merge.
- When several changes are open, the one closest to archive is named first. Accepted comes before verifying, and a tie goes to the lowest id.
- A workspace whose state says archived but that still sits in `.specsync/changes/` is still open.
- A directory under `.specsync/changes/` that holds a file or symlink but no `state.json` is still open, and the command is `specsync change status <id>`. That command says to restore state.json or remove the stray directory. A directory with no file or symlink is not an open change.
- `change ship` recommends `change ship` only for a workflow-v2 verifying change whose review verdict is pass and whose reviewed commit is HEAD. A block, a legacy record, or a review of another commit is told to run `change status`.
