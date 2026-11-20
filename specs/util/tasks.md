---
spec: util.spec.md
---

## Tasks

- [x] Document shared utility helpers in a dedicated spec
- [x] Project file functions: read, write, create and remove beneath the project root without leaving it through a link (REQ-util-002). Evidence: the five `project_file_functions_*` tests.
- [x] Read project files only through the bounded project read of `util` (REQ-util-003). Evidence: `tests/integration/project_reads.rs`.
- [x] Confine to the Git work tree, refuse absolute links and the Git directory, open writes safely, and add the walk and directory helpers (REQ-util-004, REQ-util-005). Evidence: `tests/integration/project_writes.rs` and the new `src/util.rs` tests.
- [x] Keep SpecSync's own state outside the work tree, confine to an enclosing work tree only for a project Git tracks there, and scan exactly the compiled sources (REQ-util-006, REQ-util-007, REQ-util-008). Evidence: `src/util/local_state.rs` tests, `tests/integration/project_writes.rs` and the scans.
- [x] Keep SpecSync's own state only in the user cache folder and refuse one inside the project (REQ-util-006); scan the extra opens and `std::fs` aliases (REQ-util-008). Evidence: `local_state_never_lives_in_a_git_directory`, `check_refuses_a_user_cache_folder_inside_the_project`, the read scan.

## Gaps

- `safe_regex` currently tests invalid syntax and valid patterns; oversized compiled-regex behavior depends on regex crate internals and is not covered directly.

## Post-5.0 Test Debt

- [ ] Add boundary tests for very large regex patterns if the regex crate exposes stable size-limit errors

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
