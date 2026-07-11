---
module: schema
version: 3
status: stable
files:
  - src/schema.rs
db_tables: []
tracks: [63]
depends_on: []
---

# Schema

## Purpose

Parses SQL schema files (migrations) and spec markdown to build table/column maps for bidirectional validation. Replays CREATE TABLE, ALTER TABLE, DROP TABLE, and RENAME statements in file-sorted order to produce the current schema state. Also extracts column definitions from spec `### Schema` sections for comparison.

## Public API

### Exported Structs

| Type | Description |
|------|-------------|
| `SchemaColumn` | A column extracted from SQL schema files — name, col_type (uppercase), nullable, has_default, is_primary_key |
| `SchemaTable` | All columns for a single table, built by replaying migrations in order |
| `SpecColumn` | A column documented in a spec's `### Schema` section — name and raw col_type |

### Exported SchemaTable Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `column_names` | `&self` | `Vec<&str>` | Test helper — returns column names in order (cfg(test) only) |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `build_schema` | `schema_dir: &Path` | `HashMap<String, SchemaTable>` | Build a complete schema map from SQL/migration files in the given directory, sorted by filename |
| `schema_read_errors` | `schema_dir: &Path` | `Vec<String>` | Error message for each schema/migration file that exists but cannot be read as UTF-8, plus one for a `schema_dir` that exists but cannot be enumerated (unreadable, or a file rather than a directory), so the validation gate can fail loud instead of silently under-validating (`build_schema` skips such files / returns empty) |
| `parse_spec_schema` | `body: &str` | `HashMap<String, Vec<SpecColumn>>` | Extract column definitions from a spec's `### Schema` section(s) |

## Invariants

1. `build_schema` replays migrations in filename-sorted order for deterministic results
2. `build_schema` returns an empty map if the directory does not exist
2a. A schema/migration file that exists but cannot be read as UTF-8 is silently skipped by `build_schema` (its tables/columns vanish); `schema_read_errors` reports each such file so the validation gate surfaces it as a hard error rather than letting an all-unreadable schema silently disable `db_tables`/column checks
2b. A `schema_dir` that exists but cannot be enumerated by `read_dir` — because it is unreadable or is a file rather than a directory — is itself a hard error from `schema_read_errors` (not a silent empty result): the same fail-open would otherwise leave schema discovery empty and skip `db_tables`/column checks with no signal
3. Column types are normalized to uppercase (e.g. "integer" becomes "INTEGER")
4. ALTER TABLE ADD COLUMN is idempotent — duplicate column names are skipped
5. DROP TABLE removes the table and all its columns from the map
6. ALTER TABLE RENAME TO moves all columns to the new table name
7. ALTER TABLE RENAME COLUMN preserves all column attributes except the name
8. CREATE TABLE replaces any prior definition of the same table (handles CREATE OR REPLACE semantics)
9. Table-level constraints (PRIMARY KEY, UNIQUE, CHECK, FOREIGN KEY, CONSTRAINT) are skipped during column parsing
10. String literals with escaped quotes are handled correctly during parenthesis matching
11. SQL line comments (`--`) are skipped during parenthesis matching
12. `parse_spec_schema` supports two formats: inline (`### Schema: table_name`) and multi-table (`### Schema` with `#### table_name` sub-headers)
13. `parse_spec_schema` skips markdown table header rows (column named "column")
14. Only files with recognized SQL extensions are processed (sql, ts, js, mjs, cjs, swift, kt, kts, java, py, rb, go, rs, cs, dart, php)

## Behavioral Examples

### Scenario: Build schema from migrations

- **Given** a directory with `001_create.sql` containing `CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL)` and `002_add_col.sql` containing `ALTER TABLE items ADD COLUMN price REAL DEFAULT 0`
- **When** `build_schema(dir)` is called
- **Then** returns a map with "items" having 3 columns: id (INTEGER, PK), name (TEXT, NOT NULL), price (REAL, DEFAULT)

### Scenario: DROP TABLE removes table

- **Given** SQL containing `CREATE TABLE temp (id INTEGER PRIMARY KEY)` followed by `DROP TABLE temp`
- **When** the SQL is parsed
- **Then** "temp" is not present in the resulting schema map

### Scenario: Rename table

- **Given** SQL containing `CREATE TABLE old_name (id INTEGER PRIMARY KEY)` followed by `ALTER TABLE old_name RENAME TO new_name`
- **When** the SQL is parsed
- **Then** "old_name" is absent and "new_name" has all original columns

### Scenario: Parse spec schema inline format

- **Given** a spec body with `### Schema: messages` followed by a markdown table with columns `id`, `content`, `created_at`
- **When** `parse_spec_schema(body)` is called
- **Then** returns a map with "messages" having 3 SpecColumn entries

### Scenario: Parse spec schema multi-table format

- **Given** a spec body with `### Schema` followed by `#### messages` and `#### users` sub-headers each with column tables
- **When** `parse_spec_schema(body)` is called
- **Then** returns a map with both "messages" and "users" entries

### Scenario: Nonexistent directory

- **Given** a path that does not exist
- **When** `build_schema(path)` is called
- **Then** returns an empty HashMap

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Schema directory does not exist | `build_schema` returns empty map; `schema_read_errors` returns no errors (schema simply not configured) |
| Schema directory exists but cannot be enumerated (unreadable, or a file not a directory) | `schema_read_errors` returns a hard error naming the directory (fail-loud); `build_schema` returns an empty map |
| File cannot be read | `build_schema` silently skips the file; `schema_read_errors` flags it as a hard error |
| Unmatched parentheses in CREATE TABLE | `extract_paren_body` returns `None`, table is skipped |
| No `### Schema` section in spec | `parse_spec_schema` returns empty map |
| Column name looks like SQL keyword | Column is skipped by `is_sql_keyword` check |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| regex | `Regex`, `LazyLock` for compiled SQL patterns |

### Consumed By

| Module | What is used |
|--------|-------------|
| validator | `build_schema`, `parse_spec_schema`, `SchemaTable` for column validation |
| mcp | `build_schema` for schema-aware validation |
| main | `build_schema`, `SchemaTable` for CLI schema loading |

## Change Log

| Date | Change |
|------|--------|
| 2026-07-06 | `schema_read_errors` now also fails loud when `schema_dir` exists but cannot be enumerated by `read_dir` (unreadable, or a file not a directory) — closing the same fail-open; added invariant 2b, updated the API row and Error Cases table |
| 2026-03-29 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
