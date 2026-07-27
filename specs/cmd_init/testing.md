---
spec: cmd_init.spec.md
---

# Testing

## Automated Coverage

| Area | Evidence |
|------|----------|
| Current 5.0 layout | `write_current_layout_creates_full_structure`, `fresh_init_is_not_legacy_layout`, `init_enables_sdd_for_new_projects` |
| Config safety | `init_does_not_overwrite_existing_config`, `init_does_not_overwrite_existing_v4_config` |
| Source detection | `init_auto_detects_src_dir`, `init_auto_detects_lib_dir`, `init_auto_detects_multiple_dirs`, `init_falls_back_to_src_when_no_source_files` |
| Local-state ignores | `adds_entry_to_missing_gitignore`, `is_idempotent_when_entry_already_present`, `errors_when_gitignore_path_is_unwritable` |
| MCP initialization | `mcp_tool_init_creates_config` |
| Structured truthfulness | `init_json_reports_fallback_and_complete_outcome_truthfully`, `init_plain_reinit_is_byte_identical_and_json_reports_unchanged` |
| Additive repair | `init_repair_restores_support_files_without_touching_owned_content`, `init_repair_rejects_corrupt_config_before_mutating_layout` |
| Nested/blocking safety | `init_nested_project_json_fails_without_creating_nested_metadata`, `init_preflights_blocking_layout_without_partial_writes` |

## Requirement Evidence

- `REQ-cmd-init-001`: `write_current_layout_creates_full_structure`, `fresh_init_is_not_legacy_layout`, and `init_enables_sdd_for_new_projects`.
- `REQ-cmd-init-002`: current-layout integration tests assert `.specsync/config.toml`, `.specsync/sdd.json`, and the `5.0.0` version stamp.
- `REQ-cmd-init-005`: the structured/no-op/repair/nested/blocking integration tests named above.

## Reviewer Checklist

- Run `cargo test commands::init` and the integration tests containing `init_`.
- Run `specsync init` in a clean temporary repository and inspect the complete `.specsync/` layout.
- Run the full strict spec and repository lanes before release.
