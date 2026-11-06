---
spec: cmd_init.spec.md
---

# Testing

## Automated Coverage

| Area | Evidence |
|------|----------|
| Current 5.0 layout | `write_current_layout_creates_full_structure`, `fresh_init_is_not_legacy_layout`, `fresh_init_leaves_sdd_off_so_the_first_check_is_just_drift` |
| Config safety | `init_does_not_overwrite_existing_config`, `init_does_not_overwrite_existing_v4_config` |
| Source detection | `init_auto_detects_src_dir`, `init_auto_detects_lib_dir`, `init_auto_detects_multiple_dirs`, `init_falls_back_to_src_when_no_source_files` |
| Local-state ignores | `adds_entry_to_missing_gitignore`, `is_idempotent_when_entry_already_present`, `errors_when_gitignore_path_is_unwritable` |
| MCP initialization | `mcp_tool_init_creates_config` |

## Requirement Evidence

- `REQ-cmd-init-001`: `write_current_layout_creates_full_structure`, `fresh_init_is_not_legacy_layout`, and `fresh_init_leaves_sdd_off_so_the_first_check_is_just_drift`.
- `REQ-cmd-init-002`: current-layout integration tests assert `.specsync/config.toml`, `.specsync/sdd.json`, and the `6.0.0` version stamp.
- `REQ-cmd-init-004`: `fresh_init_leaves_sdd_off_so_the_first_check_is_just_drift` (a Git repository gets both flags false), `init_then_check_is_usable_without_git_and_does_not_nag_about_legacy_layout` (a non-Git directory gets the same), and `a_policy_written_before_a_field_existed_still_loads_and_fails_closed` (omitted fields read as `true`); `init_contracts_leave_sdd_off_for_git_repositories` pins the wording.

## Reviewer Checklist

- Run `cargo test commands::init` and the integration tests containing `init_`.
- Run `specsync init` in a clean temporary repository and inspect the complete `.specsync/` layout.
- Run the full strict spec and repository lanes before release.
