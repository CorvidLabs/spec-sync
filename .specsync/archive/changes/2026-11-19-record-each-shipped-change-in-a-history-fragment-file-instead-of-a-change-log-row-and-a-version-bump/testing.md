---
change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-159 | `spec_history::two_changes_to_one_module_merge_with_plain_git_and_keep_their_history`, `spec_history::a_change_its_change_log_already_names_gets_no_fragment`, unit tests for idempotence and send-back |
| REQ-change-160 | the fragment round-trip and ordering unit tests; the `history` ordering test |
| REQ-validator-048 | `spec_history::check_refuses_a_malformed_history_fragment_and_names_it`, `spec_history::well_formed_history_fragments_pass_a_strict_check` |
| REQ-hash-cache-006 | `spec_history::a_cached_clean_check_does_not_hide_a_new_malformed_fragment` |
| REQ-cmd-change-037 | `spec_history::history_prints_the_frozen_table_then_one_row_per_fragment`, `spec_history::history_of_a_spec_without_a_change_log_uses_a_date_and_change_table`, `spec_history::history_refuses_an_unknown_module_and_a_malformed_fragment` |
| REQ-cli-args-026 | a `cli.rs` parse test |
| REQ-cli-014 | the `history` integration tests run through `main.rs` |
| REQ-change-021 | `change::tests::history_row_preserves_version_date_changes_schema`, `history_row_populates_date_author_change_schema`, `history_row_keeps_default_two_column_schema`, `history_row_does_not_treat_similar_headers_as_change_columns`, `history_versions_count_on_in_integer_or_semantic_format` |
| REQ-change-092 | `change::tests::a_materialized_spec_missing_its_history_fragment_gets_it_back`, `a_delta_corrected_after_materialization_reaches_the_canonical_spec_on_the_next_check`, `re_approving_a_byte_identical_delta_leaves_the_canonical_spec_byte_for_byte_alone`, `materialization_records_the_change_once_in_a_fragment` |
| REQ-change-123 | `change::tests::a_module_dropped_from_the_scope_is_refused_while_it_keeps_the_change_s_record`, `a_sent_back_removed_only_change_materializes_again` |

## Checks

| Check | Expected |
|---|---|
| Two changes to one module, plain `git merge`, no driver | clean merge, `version:` and Change Log unchanged, both fragments kept |
| `change check` twice | the second writes nothing |
| A change recorded by a 6.3.0 row | no fragment, no write |
| Malformed fragment, double record | `check` fails naming the file |
| `history` | frozen rows unchanged, fragments in date then id order, Version numbered on |

Locally: the full `cargo test`, clippy with `-D warnings`, fmt, `specsync check --strict --require-coverage 100 --force` (the verify lane) and the pre-push gate.
