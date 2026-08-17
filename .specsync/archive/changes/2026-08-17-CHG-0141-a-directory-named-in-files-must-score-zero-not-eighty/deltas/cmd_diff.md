## ADDED

### REQUIREMENT REQ-cmd-diff-004

`diff` SHALL treat a path that is a directory as inconclusive rather than as a file with no exports.

Acceptance Criteria
- A directory is listed among the inconclusive files, alongside paths that could not be read.
- A directory never contributes an empty export set that would read as "this file exports nothing".
