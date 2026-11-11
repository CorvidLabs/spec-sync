---
change: the-ownership-recovery-narrows-a-scope-over-files-the-change-never-touched
artifact: design
---

# Design

`ownership_preflight_blocker` keeps one blocker sentence naming every unowned input. An untouched
input's hint names its owner and says to leave it out of `affected_paths`. "Modified" means changed
since the merge base with the remote default branch, the base the coverage gate diffs against, or
untracked, and not covered by another active change. It is computed with NUL-separated
`git diff --name-only --relative --no-renames -z --end-of-options <merge-base>` and
`git ls-files --others --exclude-standard -z`. The change's `base_commit` is not diffed: after the
branch merges or rebases the default branch, that diff also holds upstream edits the change never
made. It is only a gate: it must be a full commit id, or nothing is classified as untouched and
every input keeps the declare recovery. An untouched input whose owners the printed declare answer
already declares is fixed by that answer and is not narrowed.

The recovery has three shapes:

- **Only modified inputs:** the previous declare recovery, unchanged.
- **Only untouched inputs:** a narrowing recovery. `change send-back`, then `change answer <id>
  affected_paths <paths>` listing every changed path inside the current scope, deletions included,
  so a removed file is not dropped from the scope and the shipped change still passes the
  meaningful-path audit. Scopes match as finalize matches them (`path_matches_scope`, so `.` is
  the whole project). When nothing is classified, nothing in the scope changed, a path holds a
  comma, a backslash, a control character or edge whitespace, or the list starts with `-`, the
  answer falls back to a quoted `'<path>,<path>'` placeholder, which keeps the printed command
  honest (#1011).
- **Both:** one send-back carrying the declare answer, any `files:` repair for modified inputs no
  module claims, and the narrowing answer, then one approve, check and review.
