---
spec: cmd_archive_tasks.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/archive_tasks.rs` | cargo test commands::archive_tasks | No inline tests found; add focused coverage for `cmd_archive_tasks`, `archive_tasks`, `load_config` before risky changes |

## Coverage Gaps

- Integration gap: add a fixture for "Tasks archived successfully" before changing user-visible CLI output, generated files, or error handling in cmd_archive_tasks.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Tasks archived successfully | tasks.md has 3 checked items (`- [x]`) | `cmd_archive_tasks(root, false)` is called | checked items move to `## Done` section and count is printed |
| Dry run | tasks.md has completed items | `cmd_archive_tasks(root, true)` is called | prints what would be archived without modifying files |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No tasks.md files found | Prints "nothing to archive" | Keep or add a focused assertion before changing this behavior |
| No completed tasks | Prints "nothing to archive" | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- archive-tasks --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/archive_tasks.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
