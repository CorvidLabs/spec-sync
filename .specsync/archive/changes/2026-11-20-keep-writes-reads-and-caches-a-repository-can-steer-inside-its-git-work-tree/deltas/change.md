## ADDED

### REQUIREMENT REQ-change-163

The change workflow SHALL write `change.md`, the selected artifacts, `state.json`, `approvals.json` and its other JSON records, and create its workspaces, only through the confined project write functions of `util` (REQ-util-002); an artifact SHALL be created with an exclusive create, so a dangling link is refused instead of created through. Before `change answer`, `depend`, `supersede`, `approve` and `start` write anything, they SHALL refuse, naming the file, a change whose `state.json`, `change.md`, `approvals.json` or selected artifact leads out of the repository or into the Git directory. The durable writers behind transactions, archive, retirement and late-close moves SHALL refuse every symbolic link component of the paths they write, remove or rename.

Acceptance Criteria
- `change answer` and `change approve` refuse a committed `change.md` linked outside the repository; the outside file keeps its bytes and neither the state nor the approvals change.
- `change answer` refuses a dangling artifact link and creates nothing outside.
- `change new` creates no workspace in a `.specsync/changes` linked outside the repository.
- `tests/integration/project_writes.rs` `no_project_file_is_written_outside_the_confined_helpers` finds no unmarked direct write in this module's non-test source
