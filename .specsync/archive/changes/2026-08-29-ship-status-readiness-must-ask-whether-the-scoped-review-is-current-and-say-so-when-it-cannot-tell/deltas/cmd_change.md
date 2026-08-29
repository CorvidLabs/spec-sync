## MODIFIED

### REQUIREMENT REQ-cmd-change-007

`specsync change ship-status` SHALL report ship readiness for an active change,
including local tip classification and optional live GitHub check-run trust:

- HEAD tip class: `product`, `review_only`, `archive_only`, or `other`
- verification tip presence and ancestry relative to HEAD
- whether a scoped review is recorded, and whether the recorded review is still current
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

### SPEC SECTION Invariants

1. JSON output contains no terminal coloring.
2. Domain errors always produce exit code 1.
3. `change check` runs scoped verification for one change only and fails when that verification fails; it does not rewalk archived terminal evidence.
4. `change audit` reports active-workspace and living-spec integrity only and exits non-zero on report errors.
5. `change finalize` requires current verification and scoped-review evidence and performs no provider merge.
6. `change ship-status` decides readiness from evidence CURRENCY — the recorded plan and tree still match what was verified — never from whether the recorded commit is reachable from HEAD. A squash-merge rewrites that commit, so reachability would make a squash-merged change permanently unfinalizable while its evidence is intact. The rule covers the scoped review as well as the verification: readiness asks whether the recorded review is current, reports that answer as `current`, `stale`, or `unavailable`, and treats only `current` as satisfied. An unavailable guarantee reported as a satisfied one is worse than the refusal it conceals, and readiness that never asks receives no negative answer and reads its own silence as a pass.
7. The lessons loop surfaces at each of the three moments a lesson exists: `change new` names every affected module's `specs/<module>/context.md` that holds substantive prose, a FAILED `change check` names where to record what the failure taught, and BOTH `change finalize` and `change ship` name folding the archived bundle into those specs before their remaining guidance. Every surface is a pointer, never a dump, and none can fail a lifecycle command. A passing `change check` says nothing, and a change owning no affected specs receives the same guidance it received before the fold-back existed. Both verbs also emit a `lesson_bundle` path in `--json`.
