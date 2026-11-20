## ADDED

### REQUIREMENT REQ-git-utils-012

`tracked_by_git` SHALL say whether Git tracks a path in the index or in `HEAD`, and SHALL answer no outside a repository or when Git is not installed.

Acceptance Criteria
- The transaction journal and hash cache tests see a tracked file refused, in the index and only in `HEAD`.

### REQUIREMENT REQ-git-utils-013

`output_after_child_exits_capped` SHALL keep at most the given number of stdout bytes, say whether the child wrote more, and drain the rest so the child is never left blocked on a full pipe; `output_after_child_exits` SHALL be the same with no cap.

Acceptance Criteria
- A child that writes more than the cap returns the capped bytes with the overflow reported.
