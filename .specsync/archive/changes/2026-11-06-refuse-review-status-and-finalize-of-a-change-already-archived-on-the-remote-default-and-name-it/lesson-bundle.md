# Lesson bundle: refuse-review-status-and-finalize-of-a-change-already-archived-on-the-remote-default-and-name-it

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Refuse review status and finalize of a change already archived on the remote default and name it
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: src/change/, src/change_tests.rs, src/change_tests/upstream_archive.rs, src/commands/change.rs, tests/integration.rs, tests/integration/archived_upstream.rs, CHANGELOG.md
- **Acceptance**: When the remote default branch holds an archive of the same change id and `HEAD` does not contain that archive, `change review` refuses before recording acceptance. The refusal names the change, the reference and the commit that archived it, and gives the next step: merge or rebase onto the remote default, whose archive is authoritative. The change is not moved to `accepted`.
- **Acceptance**: An archive that `HEAD` already contains is not refused: that is the reopen-and-rearchive path, which keeps working unchanged.
- **Acceptance**: For a refused change, `change status` and `change ship-status` report it as already archived upstream with the same reference, commit and next step. The handoff is never `safe` and the next action is never `finalize` or `ship`.
- **Acceptance**: `change finalize` on that change refuses with the same named cause instead of "requires exactly one trusted transition ... found 0". It still refuses: no second archive is written.
- **Acceptance**: Without an upstream archive, a change verified on a branch and squash-merged onto its base before finalize still passes `change review` and `change finalize` archives it (regression guard for #690).
- **Acceptance**: Both cases are integration tests built from the auditor's squash steps. Every other gate, including finalize's trusted-transition preflight, is unchanged.

## Evidence

- Verification commit: `f55362448808c96958ac1097aa0b9207bd356fc0`
- Base commit: `884a33d3c0cb95680dae786ff25688569ee16c96`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

Audit D1 (spec-sync#694, #690 related): after a squash before finalize, `change review` passed and recorded acceptance with handoff `safe`, then `change finalize` failed with "requires exactly one trusted transition matching its state, verification, and closing evidence; found 0". After that `check --commit` refused (accepted), `reopen` refused (evidence current), and `status` still said `safe` and pointed at finalize.

Instrumenting finalize on the auditor's fixture (main 884a33d3) showed the squash is not the cause. `archive_introduction_index` and `accepted_transition_anchors` walk `HEAD` and the remote default ref. The fixture's `origin/main` already holds `chore(lifecycle): archive greeting-version-1` from the original timeline, so `archived_package_is_in_history` is true and the working tree's closing evidence may not speak. The identical squash with `git remote remove origin` reviews and finalizes normally.

Refusing to finalize a change that is already archived upstream is correct. This change makes the refusal early and named. Leif chose this scope on 2026-11-05 18:53 ("A': named refusal"), replacing an earlier A+B choice whose reopen half would have produced a duplicate archive. The root cause is recorded on #694.

Out of scope: #694's scoped-review descendant walk (its options 1-3) and any reopen path.

## From the change's design.md

# Design

One check, shared: "is this change id already archived in reachable history while this record is not archived?" It reads the existing `archive_introduction_index` (which already walks `HEAD` and the remote default) and returns the reference and commit of the archive introduction.

- `change review`: run the check before recording a verdict. On a hit, refuse; nothing is written.
- `change status` / `change ship-status` / handoff: on a hit, readiness is not `safe`, the next action is "merge or rebase onto <reference>; its archive of <id> at <commit> is authoritative", never finalize or ship.
- `change finalize`: on a hit, refuse with the same message before the trusted-transition preflight. The preflight itself is not changed.

Message shape: "`<id>` is already archived on `<reference>` at `<commit>`; this branch holds an unarchived copy. Merge or rebase onto `<reference>`; do not finalize it again."

## From the change's testing.md

# Testing

New file `tests/integration/archived_upstream.rs`, built from the auditor's squash steps without the audit fixture: a fresh project in a temp repository, one change approved and checked on a feature branch, a bare "origin" whose default branch holds that change archived, and a local `main` reset to the base with the feature squash-merged.

1. Upstream archive present: `change review` exits non-zero, the output names the change, `origin/main` and the archive commit and the merge/rebase step, and the record is still `verifying`. `change status --json` readiness is not `safe`. `change ship-status` names the same cause. `change finalize` exits non-zero with the same cause and writes no archive.
2. No upstream archive (no remote): the same squash, then `change review` and `change finalize` succeed and the package is archived.

Case 1 must fail on main 884a33d3 before the fix. Case 2 must pass before and after.

## Results

`tests/integration/archived_upstream.rs`:

- `a_change_already_archived_on_the_remote_default_is_named_and_refused_before_acceptance` failed on main 884a33d3 (`change review` recorded a pass with handoff `safe`). With the change it passes: review, status, ship-status and finalize all name `origin/main` and the archive commit, the record stays `verifying`, and no second archive is written.
- `a_squash_before_finalize_without_an_upstream_archive_still_reviews_and_finalizes` passed before and after (#690 guard).
- `upstream_archive_is_reported_only_when_head_does_not_contain_it` (added with the 19:53 rewording): an archive HEAD contains is not reported, the same archive seen from a branch before it is reported with `origin/main` and its commit.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-127 | `a_change_already_archived_on_the_remote_default_is_named_and_refused_before_acceptance`, `a_squash_before_finalize_without_an_upstream_archive_still_reviews_and_finalizes`, `upstream_archive_is_reported_only_when_head_does_not_contain_it` |
| REQ-cmd-change-028 | `a_change_already_archived_on_the_remote_default_is_named_and_refused_before_acceptance` |

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
