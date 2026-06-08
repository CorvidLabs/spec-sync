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
