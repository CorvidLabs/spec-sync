---
spec: hooks.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/hooks.rs` (enum/parsing) | cargo test hooks:: | `hook_target_all_returns_six_targets`, `hook_target_all_contains_all_variants`, `hook_target_name_returns_expected_strings`, `hook_target_description_returns_human_readable`, `from_str_parses_all_targets`, `from_str_is_case_insensitive`, `from_str_accepts_aliases`, `from_str_returns_none_for_unknown` |
| `src/hooks.rs` (install/idempotency) | cargo test hooks:: | `install_claude_creates_file`, `install_claude_is_idempotent`, `install_claude_appends_to_existing`, `install_cursor_creates_file`, `install_copilot_creates_github_dir`, `install_agents_creates_file`, `install_precommit_creates_hook_file`, `install_precommit_appends_to_existing_hook`, `install_precommit_sets_executable_permission`, `install_claude_code_hook_creates_settings`, `install_claude_code_hook_merges_into_existing`, `install_claude_code_hook_idempotent` |
| `src/hooks.rs` (status/uninstall) | cargo test hooks:: | `is_installed_returns_false_for_empty_dir`, `is_installed_claude_detects_marker`, `uninstall_claude_removes_section`, `uninstall_claude_preserves_other_content`, `uninstall_precommit_removes_hook_file`, `uninstall_returns_false_when_not_installed`, `uninstall_claude_code_hook_is_refused`, `remove_section_deletes_file_if_empty_after`, `remove_section_stops_at_next_top_level_heading` |

## Coverage Gaps

The five `generated_hook_` regressions install the actual pre-commit hook in
disposable repositories and execute it with a fake checker that records its
arguments. Every case runs with checker exit0 and exit1. Ordinary and spaced
paths are positive controls; apostrophe, `$()` and backtick paths assert literal
project selection, preserved status, and no marker-file command execution.
Against the previous generator, the two substitution tests execute their marker
and the apostrophe test exits2; the two ordinary/spaced controls pass.

- Integration gap: add a fixture for "Install all hooks" before changing user-visible CLI output, generated files, or error handling in hooks.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Install all hooks | a project with no hooks installed | `cmd_install(root, &[])` is called | installs CLAUDE.md, .cursorrules, copilot-instructions.md, AGENTS.md, pre-commit hook, and Claude Code settings |
| Already installed | CLAUDE.md already contains "Spec-Sync Integration" | `install_hook(root, HookTarget::Claude)` is called | returns `Ok(false)` without modifying the file |
| Uninstall cursor rules | .cursorrules contains the spec-sync section | `uninstall_hook(root, HookTarget::Cursor)` is called | removes the spec-sync section, returns `Ok(true)`; deletes the file if it becomes empty |
| Check status | Claude and Precommit hooks are installed, others are not | `cmd_status(root)` is called | shows "installed" for Claude and Precommit, "not installed" for the rest |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Cannot read/write file | Returns `Err` with descriptive message | Keep or add a focused assertion before changing this behavior |
| Cannot create directory | Returns `Err` with descriptive message | Keep or add a focused assertion before changing this behavior |
| Uninstall Claude Code hook | Returns `Err` — must be removed manually | Keep or add a focused assertion before changing this behavior |
| Cannot parse existing settings.json | Returns `Err` with parse error | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/hooks.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.

## Symbolic links (REQ-hooks-006)

Every fixture is one temporary directory with the project and an `outside` directory in it, so no link in a test reaches a real file. The link tests are Unix-only.

| Case | Required behavior | Test |
|------|-------------------|------|
| A path this module writes leaves the project through a link, dangles, or is not a regular file | Refused with an error naming the path; nothing outside the project changes | `tests/integration/project_links.rs`: `hooks_install_refuses_a_target_linked_outside_the_project` (all five targets), `hooks_install_refuses_a_dangling_link_and_a_chain_that_leaves`, `hooks_install_creates_nothing_in_a_directory_linked_outside_the_project`, `hooks_install_writes_through_a_link_that_stays_inside_the_project`, `hooks_uninstall_refuses_a_target_linked_outside_the_project` |

## Bounded reads (REQ-hooks-007)

FIFOs run under a timeout and over-bound files are sparse; nothing reads `/dev/zero`. Unix-only.

| Case | Required behavior | Test |
|------|-------------------|------|
| A project file this module reads is a FIFO, a link out of the project, or over its bound | Refused without waiting or reading past the bound | `tests/integration/project_reads.rs`: `hooks_status_does_not_wait_on_a_hook_file_that_is_not_a_regular_file`; `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read`, the source scan that fails on any direct file read outside the bounded project read |

## The hooks folder (REQ-hooks-009)

Every link points inside the test's own temporary directory.

| Case | Required behavior | Test |
|------|-------------------|------|
| `core.hooksPath` inside the work tree behind a committed folder link | Refused; nothing created or made executable outside | `tests/integration/project_writes.rs`: `hooks_install_never_writes_through_a_linked_folder_on_the_hooks_path` |
| A dangling `pre-commit` link in a committed hooks folder | Refused; its target is not created | `tests/integration/project_writes.rs`: `hooks_install_never_creates_the_target_of_a_dangling_pre_commit_link` |
| A real hooks folder in the work tree | An executable `pre-commit` is installed | `tests/integration/project_writes.rs`: `hooks_install_installs_into_a_hooks_folder_in_the_work_tree` |
| A linked `pre-commit` in `.git/hooks` | Install and uninstall refuse and the target keeps its bytes | `src/hooks.rs`: `install_and_uninstall_precommit_refuse_symlinked_hook` |
