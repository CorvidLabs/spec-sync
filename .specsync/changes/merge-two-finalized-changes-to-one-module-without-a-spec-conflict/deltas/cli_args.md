## ADDED

### REQUIREMENT REQ-cli-args-023

The CLI SHALL parse `merge --install` and `merge-file` with the ancestor, current, and other paths.

Acceptance Criteria
- `merge --install` is a flag on the `merge` command.
- `merge-file` takes the ancestor path, the current path, the other path, and an optional pathname.
- Existing global flags still parse around both commands.
