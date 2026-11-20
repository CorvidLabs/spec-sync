## ADDED

### REQUIREMENT REQ-agents-013

After removing a generated artifact, `agents uninstall` SHALL remove an emptied `specsync` command or `spec-sync` skill directory only through the confined removal of `util` (REQ-util-004), never one reached through a link that leaves the project.

Acceptance Criteria
- `tests/integration/project_writes.rs` `no_project_file_is_written_outside_the_confined_helpers` finds no unmarked direct write in this module's non-test source
