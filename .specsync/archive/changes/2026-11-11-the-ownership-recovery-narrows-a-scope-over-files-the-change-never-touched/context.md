---
change: the-ownership-recovery-narrows-a-scope-over-files-the-change-never-touched
artifact: context
---

# Context

Issue #1000, found shipping #994: a change approved with an over-wide `affected_paths`
prefix (`src/`) went through approve, check and review, and ship refused it. 127 of the 164
files under the prefix belonged to modules the change never touched. The recovery then said to
declare every owning module, when the right fix was a narrower scope.

Leif's direction on 2026-11-10 (relayed by coordinator t370): rework #1000 to fit #1028. Approve
keeps skipping directory scopes, and finalize handles them. #1034 (#1002) already makes
`change review` refuse a pass while this blocker exists, so the problem surfaces before a human
records a review.

This change fixes the remaining half: `ownership_preflight_blocker` splits unowned inputs by
whether the change modified them: changed since the merge base with the remote default branch
(the base the coverage gate diffs against) or untracked, and not covered by another active change.
Modified ones keep the declare-the-owner recovery. Untouched ones get a recovery that narrows
`affected_paths`, listing the changed paths in scope. Without a full commit id as the change's
base commit, a merge base, a readable set of active changes or a complete spec listing, every
input keeps the declare recovery.

Independent reviews (2026-11-10 and 11) led to these fixes:

- A `base_commit` read from the record could become a Git option. It is now only a validity gate
  and never reaches Git.
- Non-ASCII paths, renames and edits committed before `change new` are counted as modified.
- The blocker hints agree with the recovery.
- A mixed case gets one send-back, and it keeps the `files:` repair for modified inputs no
  module claims.
- The narrowing answer lists the changed paths in scope, deletions included; forgetting a deletion
  fails `change audit --strict` once the deleted file is on the default branch (dogfooded).
- Comparing with `base_commit` counted upstream edits merged into the branch as this change's
  work; the merge base alone does not. A file another active change covers is left to it.

With no remote default branch (so no merge base), nothing is classified, and every input keeps
the declare recovery.
