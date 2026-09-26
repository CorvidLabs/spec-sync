---
change: lifecycle-commits-stage-only-what-the-change-owns-never-every-untracked-file
artifact: requirements
---

# Requirements

## `REQ-cmd-change-017` (ADDED)

Lifecycle commits stage only the project's tracked edits and the untracked paths the change owns,
through literal pathspecs. Any other untracked file stays out of every commit, and out of what
`--push` publishes, and is listed on stderr. The runtime lock and journal are neither staged nor
listed, and unmerged entries are not staged. `check --commit` commits nothing unless its first
pass verifies, and a second-pass failure names the materialize commit it leaves in place.

## `REQ-cmd-change-012` (MODIFIED)

Same guarantee: the sequence-ledger floor still runs before staging and still does not block the
author. The only change is that the text no longer says staging is `git add -A`.

## `REQ-change-102` (ADDED)

The change domain is the single answer to which untracked paths a lifecycle commit may stage, and
the answer never includes `affected_paths` prefixes or whole spec directories.

## Out of scope

- Making the project-input digest ignore untracked files. Evidence still covers the working tree
  as it sits, and the warning says what that means for a left-out file.
- Rewinding the materialize commit when the second pass fails (see design).
