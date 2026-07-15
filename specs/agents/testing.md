---
spec: agents.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/agents.rs` (enum/parsing) | cargo test agents:: | `agent_tool_all_returns_four_targets`, `agent_tool_all_contains_all_variants`, `agent_tool_name_returns_expected_strings`, `from_str_parses_all_targets`, `from_str_is_case_insensitive`, `from_str_returns_none_for_unknown` |
| `src/agents.rs` (path correctness) | cargo test agents:: | `claude_paths_are_correct`, `cursor_command_path_is_flat`, `codex_has_no_command_path`, `gemini_has_both_skill_and_command_paths` |
| `src/agents.rs` (install/idempotency) | cargo test agents:: | `install_claude_creates_skill_and_command`, `install_cursor_command_has_no_frontmatter`, `install_codex_creates_skill_only`, `install_gemini_creates_skill_and_command`, `install_is_idempotent`, `install_does_not_rewrite_unchanged_content` |
| `src/agents.rs` (content-aware reinstall) | cargo test agents:: | `install_overwrites_stale_skill_content`, `install_overwrites_stale_command_content` |
| `src/agents.rs` (status/uninstall) | cargo test agents:: | `is_installed_returns_false_for_empty_dir`, `uninstall_returns_false_when_not_installed`, `uninstall_claude_removes_skill_and_command`, `uninstall_preserves_sibling_commands`, `uninstall_cursor_flat_file_does_not_touch_commands_dir`, `uninstall_gemini_removes_skill_and_command`, `uninstall_codex_removes_skill_only` |
| `src/agents.rs` (create-spec parsing/parity) | cargo test agents:: | `create_spec_commands_classify_the_complete_remaining_input`, `checked_in_create_spec_commands_match_generated_assets`; covers `REQ-agents-003` flag-position, full-input classification, checked-in byte parity, and reinstall idempotency |

## Manual Testing

- [x] `specsync agents install` on a clean project creates all 10 artifacts (four skills plus create-spec/create-change commands for Claude/Cursor/Gemini)
- [x] `specsync agents status` correctly reflects installed/not-installed state before and after install
- [x] Simulated the create-spec and create-change instruction paths using the real scaffold/new and verified lifecycle commands
- [x] `specsync agents uninstall` with an unrelated `.claude/commands/other-command.md` present — sibling file survives, only spec-sync's namespaced subdirectory is removed
- [ ] Confirm live model discovery in real Claude Code, Cursor, Codex, and Gemini sessions; local tests currently prove installation into each tool's documented project discovery location

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Uninstall when a tool's shared `commands/` directory holds other user commands | Only the spec-sync-owned namespaced subdirectory/file is removed; the shared directory and its other contents survive |
| Re-running `agents install` after a partial install (e.g. skill written, command missing) | Fills in only the missing artifact; does not error or duplicate the existing one |
| Re-running `agents install` after upgrading spec-sync (template content changed) | Overwrites the stale artifact with current content and returns `Ok(true)`; unchanged artifacts are left alone and return `Ok(false)` |
| Cursor's flat command file is the only file in `.cursor/commands/` | Uninstall removes the file but does not remove `.cursor/commands/` itself (only namespaced `specsync/` subdirectories are cleaned up when empty) |
| Gemini's TOML `prompt` value spans multiple lines with embedded backticks/quotes in the body text | Triple-quoted TOML string (`"""..."""`) must remain balanced — verified via `content.matches("\"\"\"").count() == 2` |
