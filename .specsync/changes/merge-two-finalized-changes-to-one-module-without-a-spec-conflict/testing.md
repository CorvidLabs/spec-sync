---
change: merge-two-finalized-changes-to-one-module-without-a-spec-conflict
artifact: testing
---

# Testing

`parallel_finalized_module_changes_merge_cleanly_in_either_order` builds two branches in the shape materialize writes and runs `git merge-tree` both ways. Before the driver existed, `merge --install` was rejected and the test failed. After the fix it is clean, `version:` is 7 when both sides bumped 5 to 6, and both orders produce the same spec and requirements text.

`duplicate_requirement_id_is_refused_by_check_and_change_audit` writes `REQ-auth-002` twice. Before the fix, `check` passed. After the fix, both `check` and `change audit` exit non-zero and the output contains `REQ-auth-002`.

Unit tests on the driver cover commutative bumps, unequal version numbers, quoted versions, duplicate requirement bodies kept twice, divergent purpose left in conflict, and a one-sided edit returned byte-for-byte. The existing `merge::tests` suite still passes, including `frontmatter_version_takes_max_not_incoming`.

`merge_install_refuses_a_symlinked_gitattributes` points `.gitattributes` at another file and runs `merge --install`. The command fails and names the refusal, the link target keeps its bytes, and no `merge.specsync.driver` is recorded.

Review fixes (PR #865 Codex threads). Each test below failed on the driver before the fix and passes after it, except the three install tests, whose helper is new.

- P1: `fenced_requirement_example_never_drops_a_real_requirement`, `changelog_prefix_edit_survives_a_row_union_in_either_order`, `quoted_comma_in_an_inline_list_stays_one_item`.
- P2, merge driver: `version_scalar_syntax_divergence_stays_a_conflict`, `appended_blocks_without_a_final_newline_start_on_their_own_line`, `deleted_versus_edited_requirement_shows_the_deletion`, `escaped_pipe_in_a_changelog_cell_survives_renumbering`, `edited_ancestor_changelog_row_is_not_counted_as_new`, `fenced_changelog_heading_is_not_the_changelog`, `changelog_union_keeps_a_missing_final_newline`, `block_list_comment_is_not_part_of_the_item`, `first_ids_added_to_a_heading_free_scaffold_both_land`, `trailing_section_is_not_duplicated_by_added_blocks`.
- P2, duplicate-ID check: `an_indented_code_line_does_not_open_a_fence`, `a_fence_closes_only_on_a_bare_closing_run` (both failed first), `archived_module_duplicates_do_not_block_the_project_audit`.
- P2, install: `shell_quote_keeps_metacharacters_literal`, `attribute_lines_match_a_specs_dir_with_spaces_and_brackets` (checked with `git check-attr`), `merge_install_refuses_dry_run_and_all`.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-merge-004 | `parallel_bumps_merge_commutatively_and_count_both`, `different_version_numbers_still_merge_to_one_line`, `quoted_version_keeps_its_quotes_when_both_bumps_count`, `one_sided_edit_is_returned_unchanged`, `parallel_finalized_module_changes_merge_cleanly_in_either_order`, `frontmatter_version_takes_max_not_incoming`, `version_scalar_syntax_divergence_stays_a_conflict`, `quoted_comma_in_an_inline_list_stays_one_item`, `block_list_comment_is_not_part_of_the_item`, `changelog_prefix_edit_survives_a_row_union_in_either_order`, `escaped_pipe_in_a_changelog_cell_survives_renumbering`, `edited_ancestor_changelog_row_is_not_counted_as_new`, `fenced_changelog_heading_is_not_the_changelog`, `changelog_union_keeps_a_missing_final_newline` |
| REQ-merge-005 | `divergent_prose_stays_a_conflict`, `requirement_additions_merge_in_either_order_and_duplicate_ids_are_kept`, `fenced_requirement_example_never_drops_a_real_requirement`, `appended_blocks_without_a_final_newline_start_on_their_own_line`, `deleted_versus_edited_requirement_shows_the_deletion`, `first_ids_added_to_a_heading_free_scaffold_both_land`, `trailing_section_is_not_duplicated_by_added_blocks` |
| REQ-cmd-merge-003 | `parallel_finalized_module_changes_merge_cleanly_in_either_order`, `merge_install_refuses_a_symlinked_gitattributes`, `shell_quote_keeps_metacharacters_literal`, `attribute_lines_match_a_specs_dir_with_spaces_and_brackets` |
| REQ-cli-012 | `parallel_finalized_module_changes_merge_cleanly_in_either_order` (runs `merge --install`; `git merge-tree` invokes `merge-file` through the installed driver) |
| REQ-cli-args-023 | `parallel_finalized_module_changes_merge_cleanly_in_either_order`, `merge_install_refuses_a_symlinked_gitattributes` (both parse `merge --install --root <dir>`), `merge_install_refuses_dry_run_and_all` |
| REQ-validator-045 | `names_a_repeated_heading_and_ignores_a_fenced_example`, `a_single_heading_is_not_a_duplicate`, `duplicate_requirement_id_is_refused_by_check_and_change_audit`, `an_indented_code_line_does_not_open_a_fence`, `a_fence_closes_only_on_a_bare_closing_run` |
| REQ-change-125 | `duplicate_requirement_id_is_refused_by_check_and_change_audit`, `archived_module_duplicates_do_not_block_the_project_audit` |
