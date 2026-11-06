---
id: the-workflow-v1-cutoff-check-reports-an-unreadable-archive-tree-as-a-read-failure-not-as-an-absent-change
state: draft
type: bug_fix
base_commit: 45ef2006865ab6154d74c12ea50bea004cd46842
---

# The workflow-v1 cutoff check reports an unreadable archive tree as a read failure, not as an absent change

## Intent

The workflow-v1 cutoff check reports an unreadable archive tree as a read failure, not as an absent change

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- an_unreadable_cutoff_archive_tree_is_an_error_not_an_absent_change passes: in an offline --filter=tree:0 clone whose cutoff archive tree was never fetched, the cutoff-tree check returns an error saying the archive tree at the cutoff could not be read, instead of Ok(false) (committed failing first in 45ef2006)
- A cutoff with no archive root at all still has no archived candidates and gives the same verdict as today
- The error is never cached: only a conclusive presence is stored under the cutoff, as before
- cutoff_tree_verdict_equals_the_history_walk_verdict and every existing refusal stay unchanged; no gate or bound is weakened

## No-spec Rationale

Not applicable
