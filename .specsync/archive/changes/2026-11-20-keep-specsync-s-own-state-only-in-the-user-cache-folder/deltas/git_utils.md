## MODIFIED

### REQUIREMENT REQ-git-utils-012

`tracked_by_git` SHALL say whether Git tracks a path in the index or in `HEAD`, and SHALL answer no outside a repository or when Git is not installed. It SHALL match the path literally (`--literal-pathspecs`), so `*`, `?` and `[...]` in a folder name are never wildcards, and SHALL run Git with `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE` and the pathspec variables removed, so no environment can move or widen its answer.

Acceptance Criteria
- A project folder named `*`, unpacked where the enclosing work tree tracks other files, is not tracked, so the project is confined to itself.
