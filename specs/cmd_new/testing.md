---
spec: cmd_new.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `tests/integration/config.rs` | `cargo test --test integration config::new_` | `new_auto_detects_single_source_file`, `new_warns_when_no_source_files_match` |

## Coverage Gaps

- Integration gaps remain for `specsync new --full` companion generation and refusal to overwrite an existing spec.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Quick spec | `src/auth.rs` exists | `cmd_new(root, "auth", false)` runs | creates `specs/auth/auth.spec.md` with detected source and exports |
| Full with companions | `--full` flag | `cmd_new` runs | creates spec.md, tasks.md, context.md, requirements.md, testing.md (and design.md if `companions.design` is enabled) |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Spec already exists | Exits 1 | Keep or add a focused assertion before changing this behavior |
| No source files found | Creates spec with empty `files:` | Keep or add a focused assertion before changing this behavior |
| Dir creation fails | Exits 1 | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- new --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/new.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.

## Exact-file creation follow-up

Run `cargo test --locked --offline --bin specsync creation_source_selection` for exact root files, flat-name matching, explicit mapping precedence including missing mappings, file/directory overlap, distinct-file ambiguity, configured extension/extensionless/test/exclusion rules, escaped mappings, symlink leaves/parents and Unix named pipes. `creation_source_selection_keeps_directory_priorities_and_explicit_dot` retains each command's prior matching order; root-alias coverage includes `.`, `./`, `./.` and `.//` with exact-file overlap.

The preserved f72 optimized CLI produced `files: []` after init then new/scaffold for root lib.rs; subsequent ordinary check failed, while src/lib.rs controls bound their file. Root owns the real process regressions for all three creation commands and final full qualification. Passing draft checks do not represent authored prose or human implementation acceptance.

## Symbolic links (REQ-cmd-new-006)

Every fixture is one temporary directory with the project and an `outside` directory in it, so no link in a test reaches a real file. The link tests are Unix-only.

| Case | Required behavior | Test |
|------|-------------------|------|
| A path this module writes leaves the project through a link, dangles, or is not a regular file | Refused with an error naming the path; nothing outside the project changes | `tests/integration/project_links.rs`: `spec_creation_refuses_a_dangling_spec_link`, `spec_creation_creates_nothing_in_a_spec_directory_linked_outside_the_project`, `spec_creation_writes_through_a_spec_directory_link_that_stays_inside` (each runs `new`) |
