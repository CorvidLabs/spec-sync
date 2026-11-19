# Lesson bundle: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Record each shipped change in a history fragment file instead of a Change Log row and a version bump
- **Kind**: Feature
- **Specs**: change, cmd_change, cli_args, cli, validator, hash_cache
- **Paths**: src/change.rs, src/change/, src/change_tests.rs, src/change_tests/, src/commands/change.rs, src/commands/history.rs, src/cli.rs, src/main.rs, src/validator/, src/hash_cache.rs, tests/integration.rs, tests/integration/, specs/change/, specs/cmd_change/, specs/cli_args/, specs/cli/, specs/validator/, specs/hash_cache/, site/src/content/docs/, CHANGELOG.md, AGENTS.md, docs/HLD.md
- **Acceptance**: After change check materializes a change, each affected module has specs/<module>/history/<change-id>.md with the change id, date and title, and the spec's version: and Change Log are byte for byte unchanged; two changes to one module shipped on separate branches merge with plain git (no merge driver) without a conflict in that spec; specsync check refuses a malformed fragment and names it; specsync history <module> prints the frozen table rows unchanged followed by one row per fragment

## Evidence

- Verification commit: `928ad001166318b420ac3897f5d2f7f9a28a29cd`
- Base commit: `65864de8194c10a26f674a7a171c500ae7ba5792`
- Verified by: `specsync check --spec change --spec cli --spec cli_args --spec cmd_change --spec hash_cache --spec validator`

## From the change's context.md

# Context

spec-sync #1084, from the Chough 0.6.2 work. Every SpecSync-gated PR in CorvidLabs/chough conflicts in `specs/<module>/<module>.spec.md`, on the Change Log row and the `version:` bump that `change check` writes when it materializes a delta. Each merge makes the next PR need a local update and a hand-rebuilt history. The SpecSync merge driver (`*.spec.md merge=specsync`) cannot help, because GitHub's merge button and Update-branch never run merge drivers.

Leif approved the issue on 2026-11-18 ("Leif's decision (2026-11-18): approved.", written on the issue), and hc t4c relayed that to this agent (claude-code, hc t4t) as his scope approval for the issue's three bullets. Limits: no release, tag or publish, and no merge without his relayed yes.

What a session picking this up needs to know:

- Materialization writes three outputs per module: the delta, the `version:` bump and the Change Log row (`prepare_module_application`). The row was also the only evidence that a module's materialization ran (#741). The fragment takes over both roles. A row written by an earlier SpecSync still counts, so a change materialized before the upgrade is never recorded twice.
- The freeze needs no command and rewrites nothing: SpecSync stops writing the tables and `version:`, and `specsync history` reads a table as the frozen head of the module's history. Rows are never converted into fragments.
- Out of scope: two changes that both ADD requirements to one module still both append at the end of `requirements.md`, and plain git conflicts there. #1084 names the Change Log and `version:` only.

## From the change's design.md

# Design

- **Fragment.** `history/<change-id>.md` beside the spec (the spec's own directory, resolved through the registry-aware `canonical_module_paths`), content `---\nchange: <id>\ndate: <UTC date>\n---\n\n<title>\n`. One prepared transaction writes it with the delta's spec and requirements text, so a failed check leaves neither.
- **Recorded.** A module records a change when its fragment exists or its Change Log names `<id>:`. `prepare_module_application` and the send-back reads use that one predicate. A recorded module gets nothing more; an unrecorded one gets a fragment, never a row or a bump.
- **Canonical file.** The fragment joins the spec and companions in `lifecycle_commit_scope`, `record_covers_project_path` and the acceptance-owner resolver. Send-back finds the pre-change tree from the commit that added the fragment, falling back to the row search for older changes, and names a dropped module's fragment for removal.
- **Validation.** `validate_spec_content_with_sources` (companion pass) checks each entry of `history/`: a regular `*.md` file, frontmatter `change:` equal to the stem and a valid change id, a `YYYY-MM-DD` `date:`, one non-empty summary line; and no change both in a fragment and in a Change Log row. The snapshot digest frames every `history/` entry's name and bytes.
- **Rendering.** `specsync history <module>` (in `src/commands/history.rs`, beside `doctor` under `cmd_change`) prints the Change Log table as written, then one row per fragment in the table's columns: a Version column counts on from the frozen `version:` (integer or the patch of `major.minor.patch`), Date is the fragment's date, the description column is `<id>: <summary>` with `|` escaped. `--format json` gives `module`, `version`, `rows` and `fragments`.
- **Rejected:** converting existing rows into fragments (the issue says no row is rewritten); a freeze marker in each spec (it would touch every spec once and records nothing a reader needs); computing `version:` into the spec (that write is the conflict).

## From the change's testing.md

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

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
- `specs/cli_args/cli_args.3md`
- `specs/cli/cli.3md`
- `specs/validator/validator.3md`
- `specs/hash_cache/hash_cache.3md`
