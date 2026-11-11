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
whether the change modified them since its base commit (`git diff --name-only <base>` plus
untracked files). Modified ones keep the declare-the-owner recovery. Untouched ones get a
recovery that narrows `affected_paths`. Without a base commit or git, every input keeps the
declare recovery.

Two independent reviews (2026-11-10) led to these fixes:

- A \`base_commit\` read from the record could become a Git option; it is now validated and
  passed after \`--end-of-options\`.
- Non-ASCII paths, renames and edits committed before \`change new\` are counted as modified.
- The blocker hints agree with the recovery.
- A mixed case gets one send-back, and it keeps the \`files:\` repair for modified inputs no
  module claims.

With no remote default branch (so no merge base), nothing is classified, and every input keeps
the declare recovery.
