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

Parses SQL schema files (migrations) and spec markdown to build table/column maps for bidirectional validation. Builds one fallible schema snapshot by replaying CREATE TABLE, ALTER TABLE, DROP TABLE, and RENAME statements in exact file and statement order. Also extracts column definitions from spec `### Schema` sections for comparison.

## Public API

### Exported Structs

| Type | Description |
|------|-------------|
| `SchemaColumn` | A column extracted from SQL schema files — name, col_type (uppercase), nullable, has_default, is_primary_key |
| `SchemaTable` | All columns for a single table, built by replaying migrations in order |
| `SpecColumn` | A column documented in a spec's `### Schema` section — name and raw col_type |
| `SchemaErrorKind` | Stable category for schema directory, file, malformed-statement, missing-object, and collision failures |
| `SchemaError` | Path- and source-position-aware schema failure with kind, path, line, column, and truthful message |
| `SchemaSnapshot` | Canonical current table map plus table identities retired by DROP or RENAME |

### Exported SchemaTable Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `column_names` | `&self` | `Vec<&str>` | Test helper — returns column names in order (cfg(test) only) |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `canonicalize_table_name` | `raw: &str` | `Result<String, String>` | Parse qualified table-name segments, unescape ANSI/backtick/bracket quoting, and normalize case without conflating malformed input |
| `canonical_table_leaf` | `raw: &str` | `Result<String, String>` | Return the canonical final segment for unqualified matching without splitting dots contained inside quoted identifiers |
| `normalize_table_name` | `raw: &str` | `String` | Compatibility normalization for validator restacking; fallible replay uses `canonicalize_table_name` |
| `build_schema_snapshot` | `schema_dir: &Path` | `Result<SchemaSnapshot, SchemaError>` | Build the authoritative snapshot by replaying supported DDL in filename and byte order; fail on directory, entry, UTF-8, malformed-DDL, missing-object, and collision errors |
| `build_schema` | `schema_dir: &Path` | `HashMap<String, SchemaTable>` | Compatibility map wrapper; returns an empty map on snapshot failure and must not be used alone for validation |
| `build_schema_with_retired` | `schema_dir: &Path` | `(HashMap<String, SchemaTable>, HashSet<String>)` | Compatibility wrapper exposing retired table identities; returns empty collections on snapshot failure |
| `schema_read_errors` | `schema_dir: &Path` | `Vec<String>` | Compatibility diagnostics adapter that renders the authoritative snapshot failure, including missing/unreadable paths and malformed or inconsistent DDL |
| `parse_spec_schema` | `body: &str` | `HashMap<String, Vec<SpecColumn>>` | Extract column definitions from a spec's `### Schema` section(s) |

## Invariants

1. `build_schema_snapshot` sorts migration files and replays supported DDL in byte order within each file
2. A configured missing/unreadable schema directory, unreadable entry/file, malformed supported DDL, missing referenced object, or canonical identity collision returns `SchemaError`; it never produces a successful empty snapshot
2a. `SchemaError` identifies the source path and, for SQL replay failures, the one-based line and column
2b. `build_schema` and `build_schema_with_retired` are compatibility-only lossy wrappers; validation consumers use `build_schema_snapshot` or surface `schema_read_errors`
3. Column types are normalized to uppercase (e.g. "integer" becomes "INTEGER")
4. ALTER TABLE ADD COLUMN is idempotent — duplicate column names are skipped
5. DROP TABLE removes the table and all its columns from the map
6. ALTER TABLE RENAME TO moves all columns to the new table name
7. ALTER TABLE RENAME COLUMN preserves all column attributes except the name
8. Plain CREATE TABLE fails on a canonical duplicate, IF NOT EXISTS preserves the existing table, and OR REPLACE explicitly replaces it
8a. RENAME fails without mutation when its source is missing or its canonical target already exists; DROP fails on a missing table unless IF EXISTS is present
8b. Recreating a dropped or renamed-away table clears that identity from the retired set
9. Table-level constraints (PRIMARY KEY, UNIQUE, CHECK, FOREIGN KEY, CONSTRAINT) are skipped during column parsing
10. String literals, quoted identifiers, and escaped quotes do not create false statement boundaries or DDL matches
11. SQL line and block comments are excluded from DDL discovery and parenthesis matching
12. `parse_spec_schema` supports two formats: inline (`### Schema: table_name`) and multi-table (`### Schema` with `#### table_name` sub-headers)
13. `parse_spec_schema` skips markdown table header rows (column named "column")
14. Only files with recognized SQL extensions are processed (sql, ts, js, mjs, cjs, swift, kt, kts, java, py, rb, go, rs, cs, dart, php)
15. ANSI double-quoted, backtick-quoted, bracket-quoted, mixed-case, and qualified table references share one canonical identity parser; dots inside quoted segments remain distinct from qualification separators

## Behavioral Examples

### Scenario: Build schema from migrations

- **Given** a directory with `001_create.sql` containing `CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL)` and `002_add_col.sql` containing `ALTER TABLE items ADD COLUMN price REAL DEFAULT 0`
- **When** `build_schema_snapshot(dir)` is called
- **Then** returns a snapshot whose "items" table has 3 columns: id (INTEGER, PK), name (TEXT, NOT NULL), price (REAL, DEFAULT)

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

### Scenario: Malformed migration fails loud

- **Given** `002_broken.sql` contains `CREATE TABLE broken (id INTEGER;`
- **When** `build_schema_snapshot(path)` is called
- **Then** it returns a `MalformedStatement` error naming the file and the CREATE statement's line and column

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Configured schema directory does not exist | `build_schema_snapshot` returns `MissingDirectory`; compatibility `build_schema` returns an empty map but `schema_read_errors` exposes the failure |
| Schema directory cannot be enumerated | `build_schema_snapshot` returns `ReadDirectory` or `ReadEntry` naming the path |
| File cannot be read as UTF-8 | `build_schema_snapshot` returns `ReadFile` naming the migration |
| Unmatched parentheses or malformed supported DDL | Returns `MalformedStatement` with path, line, column, and statement preview |
| Plain CREATE collides after canonicalization | Returns `DuplicateTable`; the existing table remains unchanged |
| RENAME target collides after canonicalization | Returns `RenameCollision`; source and target remain unchanged |
| DROP/RENAME source is missing | Returns `MissingTable`, except `DROP TABLE IF EXISTS` is a no-op |
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
| validator | `build_schema_snapshot`, `canonicalize_table_name`, `canonical_table_leaf`, `parse_spec_schema`, `SchemaSnapshot`, and `SchemaTable` for existence and column validation |
| mcp | `build_schema_snapshot` for schema-aware validation |
| main | `build_schema_snapshot`, `SchemaError`, and `SchemaTable` for CLI schema loading |

## Change Log

| Date | Change |
|------|--------|
| 2026-07-26 | Added one fallible ordered schema snapshot, exact within-file DDL replay, canonical quoted/qualified identities, collision-safe mutation, and path/line diagnostics for malformed or unreadable migrations |
| 2026-07-06 | `schema_read_errors` now also fails loud when `schema_dir` exists but cannot be enumerated by `read_dir` (unreadable, or a file not a directory) — closing the same fail-open; added invariant 2b, updated the API row and Error Cases table |
| 2026-03-29 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
