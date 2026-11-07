---
change: specsync-s-own-git-history-reads-ignore-git-replace-ref-base-and-git-no-replace-objects-as-they-already-ignore-git
artifact: research
---

# Research

- `rooted_git_command` (consider_accepted_evidence_anchor.rs) strips 12 variables, including `GIT_GRAFT_FILE` and `GIT_SHALLOW_FILE`, but not `GIT_REPLACE_REF_BASE` or `GIT_NO_REPLACE_OBJECTS`.
- `git_cmd` (git_utils.rs) calls `env_clear`.
- 26 `Command::new("git")` spawns in `src/change/` inherit the full environment.
- `git replace` with `GIT_REPLACE_REF_BASE=refs/alt/` writes `refs/alt/<sha>`, and Git honours it for every read under that variable: `rev-parse HEAD^{tree}` returns the replacement's tree.
- The failing test: under the variable, `change status` exits 1 with "workflow-version history changed immutable identity from 2/Some(2) to 1/None"; the plain read exits 0.
