---
id: the-workflow-v1-cutoff-check-finds-an-archive-by-the-record-it-holds-not-only-by-a-directory-name-ending-in-its-id
state: draft
type: bug_fix
base_commit: 14903d0bb499e0ab6f2d3a83b539cacf026a1ed2
---

# The workflow-v1 cutoff check finds an archive by the record it holds, not only by a directory name ending in its id

## Intent

The workflow-v1 cutoff check finds an archive by the record it holds, not only by a directory name ending in its id

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- cutoff_tree_finds_a_workflow_v1_archive_renamed_before_the_cutoff passes: a workflow-v1 archive moved before the cutoff to a directory not ending in -<id> is accepted by the cutoff-tree check exactly as the history walk accepts it (committed failing first in 14903d0b), and the change loads
- The cutoff-tree candidates include the archived record's own resolved package path (find_change_dir), as workflow_version_state_paths does, alongside the active path and the cutoff-tree entries ending in -<id>
- cutoff_tree_verdict_equals_the_history_walk_verdict and every existing refusal stay unchanged: a record absent at the cutoff, added after it, or present only under another id is still refused, and a non-canonical -<id> entry still refuses
- The persistent cutoff cache and the warm Git-process guards are unchanged; no gate or bound is weakened

## No-spec Rationale

Not applicable
