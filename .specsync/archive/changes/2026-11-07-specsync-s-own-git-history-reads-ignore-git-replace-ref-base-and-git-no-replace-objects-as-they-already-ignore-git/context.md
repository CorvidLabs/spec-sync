---
change: specsync-s-own-git-history-reads-ignore-git-replace-ref-base-and-git-no-replace-objects-as-they-already-ignore-git
artifact: context
---

# Context

Issue #983, a Codex P2 on #979 that Leif marked as a near-term follow-up. REQ-change-148, assigned by the orc. It traces to SPEED-1.a.2 and CI-2, with no new criterion.

The issue said the read-cache and lifecycle-index guards ignore `GIT_REPLACE_REF_BASE`. While writing the failing test I found the cause is wider. The guards check only `refs/replace/`, but SpecSync's own Git processes honour `GIT_REPLACE_REF_BASE` and `GIT_NO_REPLACE_OBJECTS`:

- `rooted_git_command` already strips `GIT_GRAFT_FILE`, `GIT_SHALLOW_FILE` and the other variables that redirect or rewrite what Git walks, but not these two.
- About 26 places in the change module spawn `git` directly and inherit the whole environment, `GIT_GRAFT_FILE` included.
- `git_cmd` in `git_utils` clears the environment entirely.

So whether a read sees a replacement depends on which spawn path asks. The failing test (`3073e8ed`) shows the effect: with a replacement under `refs/alt/` that swaps HEAD for a copy whose archived record lost its workflow version, `change status` under `GIT_REPLACE_REF_BASE=refs/alt/` refuses ("changed immutable identity") while the plain read passes.
