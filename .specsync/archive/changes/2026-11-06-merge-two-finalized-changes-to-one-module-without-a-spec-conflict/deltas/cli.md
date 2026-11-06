## ADDED

### REQUIREMENT REQ-cli-012

`specsync merge-file` SHALL be dispatched as the git merge driver entry, and `specsync merge --install` SHALL be dispatched as the command that registers it.

Acceptance Criteria
- `merge-file` receives the ancestor, the current file, and the other file.
- The result is written to the current file.
- The process exits 0 only when the merge is clean.
