# Lesson bundle: specsync-s-own-git-history-reads-ignore-git-replace-ref-base-and-git-no-replace-objects-as-they-already-ignore-git

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: SpecSync's own Git history reads ignore GIT_REPLACE_REF_BASE and GIT_NO_REPLACE_OBJECTS as they already ignore GIT_GRAFT_FILE
- **Kind**: BugFix
- **Specs**: change, cli
- **Paths**: src/main.rs, src/change/consider_accepted_evidence_anchor.rs, tests/integration/change.rs, specs/change, specs/cli, CHANGELOG.md
- **Acceptance**: a_custom_replace_ref_base_does_not_change_what_status_reads passes: with a replacement under refs/alt/ that swaps HEAD for a copy whose archived record lost its workflow version, change status under GIT_REPLACE_REF_BASE=refs/alt/ gives exactly the plain read, fresh and cached, and so does GIT_NO_REPLACE_OBJECTS=1 (committed failing first in 3073e8ed)
- **Acceptance**: SpecSync clears the history-rewriting Git environment it already refuses to honour (GIT_GRAFT_FILE, GIT_SHALLOW_FILE) plus GIT_REPLACE_REF_BASE and GIT_NO_REPLACE_OBJECTS once at process start, so every Git process it spawns, configured or direct, walks the same history its read-cache and index guards check
- **Acceptance**: The read-cache and lifecycle-index guards are unchanged and still disable themselves for shallow clones, grafts and refs/replace/ refs
- **Acceptance**: No gate, bound or refusal is weakened

## Evidence

- Verification commit: `60f9bdb9dde046f91671f5006599497fb0196e7d`
- Base commit: `3073e8ed33b0d83516eda46bbf76ab8c3800a463`
- Verified by: `specsync check --spec change --spec cli`

## From the change's context.md

# Context

Issue #983, a Codex P2 on #979 that Leif marked as a near-term follow-up. REQ-change-148, assigned by the orc. It traces to SPEED-1.a.2 and CI-2, with no new criterion.

The issue said the read-cache and lifecycle-index guards ignore `GIT_REPLACE_REF_BASE`. While writing the failing test I found the cause is wider. The guards check only `refs/replace/`, but SpecSync's own Git processes honour `GIT_REPLACE_REF_BASE` and `GIT_NO_REPLACE_OBJECTS`:

- `rooted_git_command` already strips `GIT_GRAFT_FILE`, `GIT_SHALLOW_FILE` and the other variables that redirect or rewrite what Git walks, but not these two.
- About 26 places in the change module spawn `git` directly and inherit the whole environment, `GIT_GRAFT_FILE` included.
- `git_cmd` in `git_utils` clears the environment entirely.

So whether a read sees a replacement depends on which spawn path asks. The failing test (`3073e8ed`) shows the effect: with a replacement under `refs/alt/` that swaps HEAD for a copy whose archived record lost its workflow version, `change status` under `GIT_REPLACE_REF_BASE=refs/alt/` refuses ("changed immutable identity") while the plain read passes.

## From the change's design.md

# Design

**Recommended: clear the history-rewriting Git environment once, at process start.** At the top of `main`, before any thread starts, remove `GIT_REPLACE_REF_BASE`, `GIT_NO_REPLACE_OBJECTS`, `GIT_GRAFT_FILE` and `GIT_SHALLOW_FILE` from SpecSync's own environment. This is one `unsafe` block, sound because it runs single-threaded before anything reads the environment. Every Git process SpecSync spawns, configured, direct or through `git_cmd`, then walks the same history: the object graph plus `refs/replace/` and `info/grafts`. Those are exactly what the read-cache and index guards check, so the guards stay correct as written. `rooted_git_command` keeps its own list, which covers library callers that never go through `main`.

**Alternative for Leif (narrow):** add the two variables to `rooted_git_command`'s strip list only. That covers the configured path, but not the direct `Command::new("git")` spawns, which inherit them (and `GIT_GRAFT_FILE`) today.

Nothing else changes: the guards, the cache key, and every refusal.

## From the change's testing.md

# Testing

- `a_custom_replace_ref_base_does_not_change_what_status_reads` (integration; new, fails today): fresh and cached reads under `GIT_REPLACE_REF_BASE=refs/alt/`, and a fresh read under `GIT_NO_REPLACE_OBJECTS=1`, equal the plain read.
- Unchanged: `the_persistent_cache_is_off_while_replace_refs_rewrite_history`, `the_persistent_cache_is_off_while_grafts_rewrite_history`, `lifecycle_index_is_not_reused_once_a_replace_ref_rewrites_history`, and the verify lane.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-148 | integration test `a_custom_replace_ref_base_does_not_change_what_status_reads` (failing first, `3073e8ed`) |
| REQ-cli-013 | the same integration test, which runs the `specsync` binary with the variables set |

## Where these lessons go

- `specs/change/change.3md`
- `specs/cli/cli.3md`
