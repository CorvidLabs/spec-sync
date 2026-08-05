## ADDED

### REQUIREMENT REQ-change-051

Git candidate scope guards SHALL admit the tracked files that a directory
candidate expands to, treating a returned path as in scope when it equals a
candidate or is a descendant of one.

Acceptance Criteria

- A `:(top,literal)` pathspec naming a directory expands to every tracked file
  beneath it; those files are in scope because the directory requested them.
- Descendant matching compares at the path separator, so an unrelated sibling
  such as `a/bc` is never admitted by the candidate `a/b`.
- The index, modified, visibility and fsmonitor guards apply identical scope
  semantics; no guard admits a path the others would reject.
- A path sharing no candidate ancestor remains rejected, preserving the guard
  against Git returning genuinely out-of-scope paths.
- Evidence collection succeeds in a repository containing archived changes,
  which is the state of every project past its first archival.
