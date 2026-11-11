---
change: the-ownership-recovery-narrows-a-scope-over-files-the-change-never-touched
artifact: design
---

# Design

`ownership_preflight_blocker` keeps one blocker sentence naming every unowned input. An untouched
input's hint names its owner and says to leave it out of `affected_paths`. "Modified" means changed
since the change's `base_commit`, or since the merge base with the remote default branch, or
untracked. It is computed with NUL-separated `git diff --name-only --relative --no-renames -z
--end-of-options <base>` and `git ls-files --others --exclude-standard -z`. The base must be a full
commit id; otherwise nothing is classified as untouched and every input keeps the declare
recovery. An untouched input whose owners the printed declare answer already declares is fixed
by that answer and is not narrowed.

The recovery has three shapes:

- **Only modified inputs:** the previous declare recovery, unchanged.
- **Only untouched inputs:** a narrowing recovery. `change send-back`, then `change answer <id>
  affected_paths '<path>,<path>'` naming only what the change edits. The quoted placeholder keeps
  the printed command honest (#1011).
- **Both:** one send-back carrying the declare answer, any `files:` repair for modified inputs no
  module claims, and the narrowing answer, then one approve, check and review.
