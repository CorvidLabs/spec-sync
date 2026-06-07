---
spec: hooks.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/hooks.rs` | cargo test hooks:: | `hook_target_all_returns_six_targets`, `hook_target_all_contains_all_variants`, `hook_target_name_returns_expected_strings`, `hook_target_description_returns_human_readable`, `from_str_parses_all_targets`, `from_str_is_case_insensitive` |

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
