---
change: specsync-s-own-git-history-reads-ignore-git-replace-ref-base-and-git-no-replace-objects-as-they-already-ignore-git
artifact: plan
---

# Plan

## Approach

Clear the history-rewriting Git variables at process start, so every Git process SpecSync spawns walks the history its guards check (design.md). Also add the two variables to `rooted_git_command`'s strip list, for library callers.

## Out of scope

- Teaching the guards to probe custom replacement namespaces; with the environment cleared there are none to probe.
- Rewriting the 26 direct `Command::new("git")` spawns to share one builder.

## Steps

1. The failing integration test is committed (`3073e8ed`).
2. Clear the variables at the top of `main`; add them to `rooted_git_command`.
3. The test passes fresh and cached, and under `GIT_NO_REPLACE_OBJECTS`; the existing graft, shallow and replace-ref tests are unchanged.
4. CHANGELOG, `cli` and `change` change-log rows; `change check --commit`, verify, `change audit --strict`.

## Risks

- **A user who relies on `GIT_REPLACE_REF_BASE` with SpecSync** gets the plain replace namespace instead. That already happens for `GIT_GRAFT_FILE` on the configured path, and SpecSync's history verdicts should not depend on it.
- **`unsafe` `remove_var` under edition 2024.** It is sound only before other threads exist, so it is the first statement in `main`, with that justification in a comment.

## Constraints consulted

- SPEED-1.a.2, CI-2.
- The read-cache and index guards (REQ-change-140): history is fixed by HEAD when the repository is not shallow and has no grafts or `refs/replace/` refs.
- `rooted_git_command` already strips `GIT_GRAFT_FILE` and `GIT_SHALLOW_FILE`.
