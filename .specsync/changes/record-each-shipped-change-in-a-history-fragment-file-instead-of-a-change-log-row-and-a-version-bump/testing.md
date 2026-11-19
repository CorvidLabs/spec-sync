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

## Checks

| Check | Expected |
|---|---|
| Two changes to one module, plain `git merge`, no driver | clean merge, `version:` and Change Log unchanged, both fragments kept |
| `change check` twice | the second writes nothing |
| A change recorded by a 6.3.0 row | no fragment, no write |
| Malformed fragment, double record | `check` fails naming the file |
| `history` | frozen rows unchanged, fragments in date then id order, Version numbered on |

Locally: the full `cargo test`, clippy with `-D warnings`, fmt, `specsync check --strict --require-coverage 100 --force` (the verify lane) and the pre-push gate.
