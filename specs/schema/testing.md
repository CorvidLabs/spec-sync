---
spec: schema.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/schema.rs` | cargo test schema:: | `test_parse_create_table`, `test_parse_create_table_if_not_exists`, `test_parse_create_virtual_table`, `test_parse_alter_table_add_column`, `test_alter_idempotent`, `test_table_constraints_skipped` |

## Coverage Gaps

- Integration gap: add a fixture for "Build schema from migrations" before changing user-visible CLI output, generated files, or error handling in schema.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Build schema from migrations | a directory with `001_create.sql` containing `CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL)` and `002_add_col.sql` containing `ALTER TABLE items ADD COLUMN price REAL DEFAULT 0` | `build_schema(dir)` is called | returns a map with "items" having 3 columns: id (INTEGER, PK), name (TEXT, NOT NULL), price (REAL, DEFAULT) |
| DROP TABLE removes table | SQL containing `CREATE TABLE temp (id INTEGER PRIMARY KEY)` followed by `DROP TABLE temp` | the SQL is parsed | "temp" is not present in the resulting schema map |
| Rename table | SQL containing `CREATE TABLE old_name (id INTEGER PRIMARY KEY)` followed by `ALTER TABLE old_name RENAME TO new_name` | the SQL is parsed | "old_name" is absent and "new_name" has all original columns |
| Parse spec schema inline format | a spec body with `### Schema: messages` followed by a markdown table with columns `id`, `content`, `created_at` | `parse_spec_schema(body)` is called | returns a map with "messages" having 3 SpecColumn entries |
| Parse spec schema multi-table format | a spec body with `### Schema` followed by `#### messages` and `#### users` sub-headers each with column tables | `parse_spec_schema(body)` is called | returns a map with both "messages" and "users" entries |
| Nonexistent directory | a path that does not exist | `build_schema(path)` is called | returns an empty HashMap |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Schema directory does not exist | `build_schema` returns empty map | Keep or add a focused assertion before changing this behavior |
| File cannot be read | File is silently skipped | Keep or add a focused assertion before changing this behavior |
| Unmatched parentheses in CREATE TABLE | `extract_paren_body` returns `None`, table is skipped | Keep or add a focused assertion before changing this behavior |
| No `### Schema` section in spec | `parse_spec_schema` returns empty map | Keep or add a focused assertion before changing this behavior |
| Column name looks like SQL keyword | Column is skipped by `is_sql_keyword` check | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/schema.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
