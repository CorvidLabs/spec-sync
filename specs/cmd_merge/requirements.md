---
spec: cmd_merge.spec.md
---

## User Stories

- As a developer, I want the `cmd_merge` module to work reliably so that spec-sync validation and tooling is trustworthy
- As a CI operator, I want clear exit codes and error messages so that pipeline failures are actionable

## Acceptance Criteria

- All exported functions perform their documented purpose
- Error conditions produce clear, actionable messages
- Module follows the project's established patterns for config loading and output formatting

## Constraints

- Must not panic on expected error conditions — return Results or print and exit
- Must work with the project's Clap-based CLI argument parsing

## Out of Scope

- GUI or web interface
- Interactive prompts (except wizard module)

### REQ-cmd-merge-001

The merge command SHALL delegate conservative spec-conflict resolution and SHALL report resolved, skipped, and failed files accurately.

Acceptance Criteria
- All exported functions perform their documented purpose
- Error conditions produce clear, actionable messages
- Module follows the project's established patterns for config loading and output formatting


### REQ-cmd-merge-002

`merge` SHALL exit non-zero when its scan could not run.

Acceptance Criteria
- A scan that could not be performed exits non-zero rather than reporting that no conflicts need resolution.
- An unperformed scan is never reported as a pass.

### REQ-cmd-merge-003

`specsync merge --install` SHALL register the git merge driver for this clone, and `specsync merge-file` SHALL write a clean contract merge or leave a conflict.

Acceptance Criteria
- `--install` appends `merge=specsync` rules for `*.spec.md` and `requirements.md` under the configured specs directory and sets `merge.specsync.driver` to this executable.
- A symlinked `.gitattributes` is refused.
- `merge-file` writes the merged bytes to the current file and exits 0 only when the merge is clean.
- `cmd_merge`'s marker scan is unchanged.

### REQ-cmd-merge-004

When the configured specs directory is the repository root, `specsync merge --install` SHALL emit attribute patterns that Git matches for root and nested contracts.

Acceptance Criteria
- `specs_dir = "."` and `specs_dir = "./"` produce rules for `*.spec.md` and `requirements.md` without a `./` directory prefix.
- `git check-attr merge` reports `merge: specsync` for root and nested `.spec.md` and `requirements.md` paths.
- Existing non-root directory normalization, wildcard escaping, local driver configuration, symlink refusal, and failure rollback remain unchanged.

### REQ-cmd-merge-005

`merge` SHALL read `.gitattributes` (4 MiB) and the spec files it resolves (8 MiB) only through the bounded project read of `util` (REQ-util-003), with a bound of the one named for each file. A read refused there, because the file leaves the project through a symbolic link, is a dangling link, is not a regular file, or is larger than the bound, SHALL be handled as that read's other read errors are, with a message that names the path and the reason, and SHALL never wait on the file or read past the bound. The files Git hands the merge driver are not project files and are out of scope.

Acceptance Criteria
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no direct file read in this module's non-test source outside the allowed list

### REQ-cmd-merge-006

`merge --install` and the merge driver command SHALL write project files only through the confined project write functions of `util` (REQ-util-002), or through a write whose path was refused as a link just before; any other direct write SHALL say, on the write or the line above it in the same statement, why its path is not a project file (a temporary or Git directory, a test barrier, a file Git hands the command). It SHALL update or restore `.gitattributes` through the confined project write.

Acceptance Criteria
- `tests/integration/project_writes.rs` `no_project_file_is_written_outside_the_confined_helpers` finds no unmarked direct write in this module's non-test source

