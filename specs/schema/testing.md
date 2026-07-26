---
spec: schema.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/schema.rs` | `fledge run test -- schema::tests` | 36 focused tests, especially the ordered-replay, malformed-SQL, quoting, missing-object, and collision tests |
| `tests/integration/config.rs` | `fledge run test -- --test integration config::check_gates_on_unreadable_schema_file` | CLI fails closed and renders the snapshot's unreadable-migration diagnostic |

## Coverage Gaps

- Validator/CLI consumption is intentionally deferred to the #453 restack: it must consume `build_schema_snapshot` directly and must not infer validity from an empty compatibility map.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Build schema from migrations | a directory with ordered CREATE and ALTER files | `build_schema_snapshot(dir)` is called | returns a snapshot with the final canonical table and column state |
| Exact statement order | one file performs CREATE, DROP, then CREATE of the same name | SQL is replayed | the final CREATE exists and the identity is no longer retired |
| Rename then recreate | one file creates `users`, renames it to `archived_users`, then creates `users` again | SQL is replayed | both final tables exist with their own columns |
| Malformed supported DDL | `002_broken.sql` contains an unmatched CREATE TABLE parenthesis | `build_schema_snapshot(dir)` is called | returns `MalformedStatement` with file, line, column, and statement preview |
| Quoted and qualified identity | ANSI, backtick, bracket, mixed-case, and qualified names—including a dot inside a quoted segment—are normalized | SQL is replayed | each operation resolves through the same canonical identity without conflating quoted dots with qualification |
| Collision safety | normalized CREATE names collide, or RENAME targets an existing canonical table | SQL is replayed | returns `DuplicateTable`/`RenameCollision` without overwriting either existing table |
| Parse spec schema inline format | a spec body with `### Schema: messages` followed by a markdown table with columns `id`, `content`, `created_at` | `parse_spec_schema(body)` is called | returns a map with "messages" having 3 SpecColumn entries |
| Parse spec schema multi-table format | a spec body with `### Schema` followed by `#### messages` and `#### users` sub-headers each with column tables | `parse_spec_schema(body)` is called | returns a map with both "messages" and "users" entries |
| Configured missing directory | a path that does not exist | `build_schema_snapshot(path)` is called | returns `MissingDirectory`; no successful empty snapshot is produced |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Schema directory does not exist | `build_schema_snapshot` returns `MissingDirectory` | `test_build_schema_snapshot_missing_directory_is_error` |
| File cannot be read as UTF-8 | `build_schema_snapshot` returns `ReadFile` naming the file | `test_schema_read_errors_flags_only_unreadable` |
| Unmatched parentheses in CREATE TABLE | `build_schema_snapshot` returns a positioned `MalformedStatement` | `test_build_schema_snapshot_reports_malformed_sql_with_path_and_position` |
| Missing terminator before later DDL | snapshot fails before applying the first statement | `test_replay_rejects_unterminated_statement_before_later_ddl` |
| DDL text in comments/string literals | ignored as operations | `test_replay_ignores_commented_and_string_literal_ddl` |
| DROP then CREATE / RENAME then CREATE | exact source order determines final state | `test_replay_respects_drop_then_recreate_within_one_file`, `test_replay_respects_recreate_after_rename_within_one_file` |
| CREATE canonical collision | existing table is preserved and `DuplicateTable` is returned | `test_replay_duplicate_canonical_create_is_a_non_mutating_error` |
| RENAME target collision | source and target are preserved and `RenameCollision` is returned | `test_replay_rename_collision_is_a_non_mutating_error` |
| Missing DROP/RENAME source | hard error except DROP IF EXISTS | `test_replay_missing_drop_and_rename_sources_fail_truthfully` |
| Quoted/qualified CREATE→RENAME→DROP | one canonical parser governs every transition | `test_quoted_qualified_names_replay_through_rename_and_drop` |
| No `### Schema` section in spec | `parse_spec_schema` returns empty map | Keep or add a focused assertion before changing this behavior |
| Column name looks like SQL keyword | Column is skipped by `is_sql_keyword` check | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/schema.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
