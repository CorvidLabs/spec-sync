---
change: specsync-s-own-git-history-reads-ignore-git-replace-ref-base-and-git-no-replace-objects-as-they-already-ignore-git
artifact: testing
---

# Testing

- `a_custom_replace_ref_base_does_not_change_what_status_reads` (integration; new, fails today): fresh and cached reads under `GIT_REPLACE_REF_BASE=refs/alt/`, and a fresh read under `GIT_NO_REPLACE_OBJECTS=1`, equal the plain read.
- Unchanged: `the_persistent_cache_is_off_while_replace_refs_rewrite_history`, `the_persistent_cache_is_off_while_grafts_rewrite_history`, `lifecycle_index_is_not_reused_once_a_replace_ref_rewrites_history`, and the verify lane.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-148 | integration test `a_custom_replace_ref_base_does_not_change_what_status_reads` (failing first, `3073e8ed`) |
| REQ-cli-013 | the same integration test, which runs the `specsync` binary with the variables set |
