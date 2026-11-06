---
id: specsync-s-own-git-history-reads-ignore-git-replace-ref-base-and-git-no-replace-objects-as-they-already-ignore-git
state: implementing
type: bug_fix
base_commit: 3073e8ed33b0d83516eda46bbf76ab8c3800a463
---

# SpecSync's own Git history reads ignore GIT_REPLACE_REF_BASE and GIT_NO_REPLACE_OBJECTS as they already ignore GIT_GRAFT_FILE

## Intent

SpecSync's own Git history reads ignore GIT_REPLACE_REF_BASE and GIT_NO_REPLACE_OBJECTS as they already ignore GIT_GRAFT_FILE

## Affected Canonical Specs

- `change`
- `cli`

## Acceptance Criteria

- a_custom_replace_ref_base_does_not_change_what_status_reads passes: with a replacement under refs/alt/ that swaps HEAD for a copy whose archived record lost its workflow version, change status under GIT_REPLACE_REF_BASE=refs/alt/ gives exactly the plain read, fresh and cached, and so does GIT_NO_REPLACE_OBJECTS=1 (committed failing first in 3073e8ed)
- SpecSync clears the history-rewriting Git environment it already refuses to honour (GIT_GRAFT_FILE, GIT_SHALLOW_FILE) plus GIT_REPLACE_REF_BASE and GIT_NO_REPLACE_OBJECTS once at process start, so every Git process it spawns, configured or direct, walks the same history its read-cache and index guards check
- The read-cache and lifecycle-index guards are unchanged and still disable themselves for shallow clones, grafts and refs/replace/ refs
- No gate, bound or refusal is weakened

## No-spec Rationale

Not applicable
