---
module: schema
version: 4
status: stable
files:
  - src/schema.rs
db_tables: []
tracks: [63]
depends_on: []
---

# Schema

## Purpose

Parses SQL migrations and spec Markdown into canonical table/column identities for bidirectional
validation. One fallible snapshot replays supported DDL in exact statement order across
filename-sorted inputs; compatibility wrappers remain available for older callers.

## Public API

### Exported Structs

| Type | Description |
|------|-------------|
| `SchemaColumn` | A column extracted from SQL schema files — name, col_type (uppercase), nullable, has_default, is_primary_key |
| `SchemaTable` | All columns for a single table, built by replaying migrations in order |
| `SpecColumn` | A column documented in a spec's `### Schema` section — name and raw col_type |
| `SchemaSnapshot` | Crate-private checked tables, retired canonical identities, and replay sources |
| `SchemaErrorKind` | Crate-private typed read, parse, missing-object, duplicate, and collision failures |
| `SchemaError` | Crate-private path-aware failure with one-based source coordinates |

### Exported SchemaTable Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `column_names` | `&self` | `Vec<&str>` | Test helper — returns column names in order (cfg(test) only) |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `build_schema` | `schema_dir: &Path` | `HashMap<String, SchemaTable>` | Compatibility wrapper over ordered replay; returns an empty map when the checked snapshot fails |
| `build_schema_snapshot` | `schema_dir: &Path` | `Result<SchemaSnapshot, SchemaError>` | Build the single deterministic fallible replay snapshot used by validation |
| `canonicalize_table_name` | `raw: &str` | `Result<String, String>` | Normalize a quoted or qualified table identity |
| `canonical_table_leaf` | `raw: &str` | `Result<String, String>` | Return the canonical final segment of a table identity |
| `table_reference_matches` | `declaration, discovered` | `Result<bool, String>` | Match full qualified declarations or unqualified canonical leaves |
| `pattern_table_names` | Alias for `SchemaSnapshot::pattern_table_names` | `Result<HashSet<String>, String>` | Supplement replay with configured captures without resurrecting retired names |
| `schema_read_errors` | `schema_dir: &Path` | `Vec<String>` | Return the checked snapshot's path-aware replay/read error for validation callers |
| `parse_spec_schema` | `body: &str` | `HashMap<String, Vec<SpecColumn>>` | Extract column definitions from a spec's `### Schema` section(s) |

## Invariants

1. Checked replay processes filename-sorted migrations and preserves statement order within a file.
2. Missing, unreadable, non-UTF-8, malformed, or semantically invalid configured input returns one
   path-aware error with a one-based line/column and bounded statement preview.
3. Column types are normalized to uppercase (e.g. "integer" becomes "INTEGER")
4. Plain duplicate CREATE/ADD operations fail; `IF NOT EXISTS` preserves existing state and
   `OR REPLACE` explicitly replaces a table definition.
5. DROP TABLE removes the table and all its columns from the map
6. ALTER TABLE RENAME TO moves all columns to the new table name
7. ALTER TABLE RENAME COLUMN preserves all column attributes except the name
8. ANSI, backtick, bracket, mixed-case, quoted-dot, and qualified table names share one canonical
   identity; canonical table/column collisions fail before replacement unless explicitly allowed.
9. Table-level constraints (PRIMARY KEY, UNIQUE, CHECK, FOREIGN KEY, CONSTRAINT) are skipped during column parsing
10. String literals with escaped quotes cannot terminate statements or introduce phantom DDL.
11. SQL line/block comments and quoted content cannot introduce false operations or boundaries.
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
| 2026-07-30 | CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara: Stabilize SpecSync 6.0 with one scope approval, same-PR finalization, lightweight archive CI, scoped review, and selected UX fixes |
