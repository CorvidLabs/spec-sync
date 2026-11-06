# Lesson bundle: merge-two-finalized-changes-to-one-module-without-a-spec-conflict

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Merge two finalized changes to one module without a spec conflict
- **Kind**: Feature
- **Specs**: merge, cmd_merge, validator, change, cli, cli_args
- **Paths**: src/merge_driver.rs, src/commands/merge.rs, src/main.rs, src/cli.rs, src/validator.rs, src/validator/consumed_by_re.rs, src/validator/requirement_ids.rs, src/change/historical_definition_digest_matches.rs, tests/integration.rs, tests/integration/parallel_module_merge.rs, hi/merge.md, .gitattributes, specs/merge/merge.spec.md, specs/merge/merge.3md, specs/merge/requirements.md, specs/merge/tasks.md, specs/cmd_merge/cmd_merge.spec.md, specs/cmd_merge/cmd_merge.3md, specs/cmd_merge/requirements.md, specs/validator/validator.spec.md, specs/validator/validator.3md, specs/validator/requirements.md, specs/change/change.spec.md, specs/change/change.3md, specs/change/requirements.md, specs/cli/cli.spec.md, specs/cli/cli.3md, specs/cli_args/cli_args.spec.md, specs/cli_args/cli_args.3md, specs/cli/requirements.md, specs/cli_args/requirements.md
- **Acceptance**: MERGE-1: Two independent changes to the same module, each finalized on its own branch, merge cleanly in either order with no conflict on that module's .spec.md (version:, change-log row, files:) or on requirements.md.
- **Acceptance**: MERGE-2: A fixture with two such branches proves it with git merge-tree (failing first, then green).
- **Acceptance**: MERGE-3: After a merge that leaves the same requirement ID twice, both check and change audit refuse and name the duplicate ID.
- **Acceptance**: MERGE-4: Existing strictness stays: no looser gates, timeouts, or weakened assertions.

## Evidence

- Verification commit: `c36e0d7a2638cb7ef8648e0f18b1b43db7e0fcec`
- Base commit: `40dcf33a6b0f0b55dcb7f17e8e98be9d55fa2c56`
- Verified by: `specsync check --spec change --spec cli --spec cli_args --spec cmd_merge --spec merge --spec validator`

## From the change's context.md

# Context

Issue #862: two branches that each finalize a change to the same module conflict on that module's spec. Each materialize writes `version: N+1`, one change-log row, any new `files:` entries, and a new requirement block at the same place, so the second merge conflicts.

Leif confirmed MERGE-1 through MERGE-4 on 2026-11-05. Those sentences are in `hi/merge.md`. They are the only acceptance criteria.

The marker resolver in `src/merge.rs` still takes `max(version)` and still refuses prose, because a conflict hunk has no ancestor. The new driver in `src/merge_driver.rs` has the ancestor, so it can count both bumps. `specsync merge --install` registers it for `git merge-tree`. GitHub's merge button does not run a custom driver.

This change is waiting on Leif to record `change approve` and `change review`. Those were not recorded here.

## From the change's design.md

# Design

Git calls the driver with the ancestor, the current file, and the other file. The result is written back to the current file. Exit 0 means clean. Exit 1 leaves conflict markers in place.

For a module spec, frontmatter lists `files`, `db_tables`, and `depends_on` are unioned and sorted. Numeric `version` becomes ancestor plus both deltas, never below either side. The change log unions rows and sorts them by the first cell, then the row text. Other prose that both sides changed differently stays a conflict.

For `requirements.md`, blocks are keyed by `### REQ-...`. An ID added on only one side is kept. The same new ID with two bodies is kept twice. An existing block edited on both sides stays a conflict.

`check` and `change audit` scan living `requirements.md` files and name any ID that appears more than once outside a fence.

## From the change's testing.md

# Testing

`parallel_finalized_module_changes_merge_cleanly_in_either_order` builds two branches in the shape materialize writes and runs `git merge-tree` both ways. Before the driver existed, `merge --install` was rejected and the test failed. After the fix it is clean, `version:` is 7 when both sides bumped 5 to 6, and both orders produce the same spec and requirements text.

`duplicate_requirement_id_is_refused_by_check_and_change_audit` writes `REQ-auth-002` twice. Before the fix, `check` passed. After the fix, both `check` and `change audit` exit non-zero and the output contains `REQ-auth-002`.

Unit tests on the driver cover commutative bumps, unequal version numbers, quoted versions, duplicate requirement bodies kept twice, divergent purpose left in conflict, and a one-sided edit returned byte-for-byte. The existing `merge::tests` suite still passes, including `frontmatter_version_takes_max_not_incoming`.

`merge_install_refuses_a_symlinked_gitattributes` points `.gitattributes` at another file and runs `merge --install`. The command fails and names the refusal, the link target keeps its bytes, and no `merge.specsync.driver` is recorded.

Review fixes (PR #865 Codex threads). Each test below failed on the driver before the fix and passes after it, except the three install tests, whose helper is new.

- P1: `fenced_requirement_example_never_drops_a_real_requirement`, `changelog_prefix_edit_survives_a_row_union_in_either_order`, `quoted_comma_in_an_inline_list_stays_one_item`.
- P2, merge driver: `version_scalar_syntax_divergence_stays_a_conflict`, `appended_blocks_without_a_final_newline_start_on_their_own_line`, `deleted_versus_edited_requirement_shows_the_deletion`, `escaped_pipe_in_a_changelog_cell_survives_renumbering`, `edited_ancestor_changelog_row_is_not_counted_as_new`, `fenced_changelog_heading_is_not_the_changelog`, `changelog_union_keeps_a_missing_final_newline`, `block_list_comment_is_not_part_of_the_item`, `first_ids_added_to_a_heading_free_scaffold_both_land`, `trailing_section_is_not_duplicated_by_added_blocks`.
- P2, duplicate-ID check: `an_indented_code_line_does_not_open_a_fence`, `a_fence_closes_only_on_a_bare_closing_run` (both failed first), `archived_module_duplicates_do_not_block_the_project_audit`.
- Second review round (2906dc30), each failed first: `requirements_union_keeps_the_current_final_newline`, `indented_requirement_heading_is_a_block`, `indented_atx_requirement_headings_are_counted`, `dot_prefixed_specs_dir_matches_the_contracts`.
- Third review round (d2973cf1), each failed first: `deleted_versus_edited_frontmatter_field_shows_the_deletion`, `first_version_rows_on_an_empty_table_are_not_left_equal`, `failed_install_leaves_gitattributes_unchanged`, `config_failure_restores_gitattributes`.
- P2, install: `shell_quote_keeps_metacharacters_literal`, `attribute_lines_match_a_specs_dir_with_spaces_and_brackets` (checked with `git check-attr`), `merge_install_refuses_dry_run_and_all`.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-merge-004 | `parallel_bumps_merge_commutatively_and_count_both`, `different_version_numbers_still_merge_to_one_line`, `quoted_version_keeps_its_quotes_when_both_bumps_count`, `one_sided_edit_is_returned_unchanged`, `parallel_finalized_module_changes_merge_cleanly_in_either_order`, `frontmatter_version_takes_max_not_incoming`, `version_scalar_syntax_divergence_stays_a_conflict`, `quoted_comma_in_an_inline_list_stays_one_item`, `block_list_comment_is_not_part_of_the_item`, `changelog_prefix_edit_survives_a_row_union_in_either_order`, `escaped_pipe_in_a_changelog_cell_survives_renumbering`, `edited_ancestor_changelog_row_is_not_counted_as_new`, `fenced_changelog_heading_is_not_the_changelog`, `changelog_union_keeps_a_missing_final_newline`, `deleted_versus_edited_frontmatter_field_shows_the_deletion`, `first_version_rows_on_an_empty_table_are_not_left_equal` |
| REQ-merge-005 | `divergent_prose_stays_a_conflict`, `requirement_additions_merge_in_either_order_and_duplicate_ids_are_kept`, `fenced_requirement_example_never_drops_a_real_requirement`, `appended_blocks_without_a_final_newline_start_on_their_own_line`, `deleted_versus_edited_requirement_shows_the_deletion`, `first_ids_added_to_a_heading_free_scaffold_both_land`, `trailing_section_is_not_duplicated_by_added_blocks`, `requirements_union_keeps_the_current_final_newline`, `indented_requirement_heading_is_a_block` |
| REQ-cmd-merge-003 | `parallel_finalized_module_changes_merge_cleanly_in_either_order`, `merge_install_refuses_a_symlinked_gitattributes`, `shell_quote_keeps_metacharacters_literal`, `attribute_lines_match_a_specs_dir_with_spaces_and_brackets`, `dot_prefixed_specs_dir_matches_the_contracts`, `failed_install_leaves_gitattributes_unchanged`, `config_failure_restores_gitattributes` |
| REQ-cli-012 | `parallel_finalized_module_changes_merge_cleanly_in_either_order` (runs `merge --install`; `git merge-tree` invokes `merge-file` through the installed driver) |
| REQ-cli-args-023 | `parallel_finalized_module_changes_merge_cleanly_in_either_order`, `merge_install_refuses_a_symlinked_gitattributes` (both parse `merge --install --root <dir>`), `merge_install_refuses_dry_run_and_all` |
| REQ-validator-045 | `names_a_repeated_heading_and_ignores_a_fenced_example`, `a_single_heading_is_not_a_duplicate`, `duplicate_requirement_id_is_refused_by_check_and_change_audit`, `an_indented_code_line_does_not_open_a_fence`, `a_fence_closes_only_on_a_bare_closing_run`, `indented_atx_requirement_headings_are_counted` |
| REQ-change-125 | `duplicate_requirement_id_is_refused_by_check_and_change_audit`, `archived_module_duplicates_do_not_block_the_project_audit` |

## Where these lessons go

- `specs/merge/merge.3md`
- `specs/cmd_merge/cmd_merge.3md`
- `specs/validator/validator.3md`
- `specs/change/change.3md`
- `specs/cli/cli.3md`
- `specs/cli_args/cli_args.3md`
