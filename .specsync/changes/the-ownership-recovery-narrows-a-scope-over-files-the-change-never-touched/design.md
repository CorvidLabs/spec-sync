---
change: the-ownership-recovery-narrows-a-scope-over-files-the-change-never-touched
artifact: design
---

# Design

`ownership_preflight_blocker` keeps one blocker sentence naming every unowned input. Its
recovery is assembled from two parts. `declare_owners_recovery` is the previous recovery,
unchanged, applied to modified inputs. `narrow_scope_recovery` names up to five untouched inputs
("and N more") and prints `change send-back`, then `change answer <id> affected_paths <paths>`.
The `<paths>` placeholder keeps the printed command honest (#1011): the tool does not guess
the narrowed scope. "Modified" means changed between the change's `base_commit` and the working
tree, or untracked. That is the same evidence a reviewer sees in the PR diff.
