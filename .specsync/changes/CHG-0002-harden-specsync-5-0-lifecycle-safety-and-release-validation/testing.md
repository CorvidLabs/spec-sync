---
change: CHG-0002-harden-specsync-5-0-lifecycle-safety-and-release-validation
artifact: testing
---

# Testing

Each review finding gets a minimal regression plus the full existing suite. Adversarial coverage includes last-block Markdown edits, dirty worktrees, malformed policy, phantom effective APIs, late conflicts, reverse-ID dependencies, failed evidence, prefix-collision paths, shallow/detached/non-main Git states, concurrent IDs/acceptance, interrupted writes, symlinks, Unicode/spaces/CRLF/case behavior, large artifacts, and custom-policy combinations. Release proof uses packaged binaries, clean consumer repositories, the Action, imported real-format fixtures, and installed Claude/Cursor/Codex/Gemini surfaces on supported platforms.

## Requirement Evidence

- `REQ-change-005`: `markdown_block_stops_at_higher_level_heading`, `markdown_block_preserves_crlf_and_unrelated_bytes`, `prepared_write_failure_rolls_back_prior_files`, and `pending_transaction_is_recovered_before_next_lifecycle_write`.
- `REQ-change-006`: `working_tree_changes_invalidate_verification`, `workspace_digest_tracks_unicode_and_space_paths`, and `failed_verification_evidence_keeps_unified_check_red`.
- `REQ-change-007`: `malformed_policy_fails_closed`, `unavailable_required_path_coverage_fails_closed`, `oversized_change_artifacts_are_rejected`, `safe_project_paths_reject_symlink_escapes`, and `unified_gate_validates_code_against_effective_delta`.
- `REQ-change-008`: `dependent_changes_are_topologically_ordered`, `acceptance_rechecks_late_dependency_state`, `path_scopes_match_component_boundaries`, `concurrent_change_creation_assigns_unique_ids`, and `clean_feature_branch_still_requires_changed_path_coverage`.
- `REQ-change-009`: `definition_approval_rejects_an_invalid_semantic_delta`, `requirement_ids_must_match_their_delta_module`, `malformed_active_change_state_fails_closed`, `failed_archive_move_leaves_an_accepted_change_retryable`, `draft_requirement_removals_are_not_permanent_tombstones`, and `speckit_adoption_imports_constitution_and_feature_workspaces_only`.
- `REQ-change-010`: `default_policy_covers_root_action_and_dependency_lockfiles` and `path_scopes_match_component_boundaries`.
- `REQ-cmd-init-001`: `write_current_layout_creates_full_structure`, `fresh_init_is_not_legacy_layout`, and `init_enables_sdd_for_new_projects`.
