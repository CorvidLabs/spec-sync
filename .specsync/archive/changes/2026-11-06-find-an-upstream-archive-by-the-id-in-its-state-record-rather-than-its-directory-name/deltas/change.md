## ADDED

### REQUIREMENT REQ-change-139

The upstream-archive lookup SHALL identify the remote default's archive of a change by the `id` recorded in each archived package's `state.json`, as `find_change_dir` does, not by the package directory name, and SHALL do so with a bounded number of Git processes that does not grow with the number of archived packages.

Acceptance Criteria
- A renamed or undated upstream package of the change is found.
- The lookup uses one tree listing and one batched blob read of the archive directory, plus per-match work only for a matching package.
