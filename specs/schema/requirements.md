---
spec: schema.spec.md
---

## User Stories

- As a developer with database migrations, I want spec-sync to parse my SQL migrations and validate that my spec's Schema section matches the actual database schema so that schema documentation stays accurate
- As a developer, I want migrations replayed in filename-sorted order so that the final schema state is deterministic regardless of filesystem ordering
- As a developer, I want ALTER TABLE statements (ADD COLUMN, DROP COLUMN, RENAME) handled so that the schema reflects the full migration history, not just CREATE TABLE
- As a developer writing specs, I want to document schemas in markdown tables so that the format is readable and diffable

## Acceptance Criteria

- `build_schema` replays CREATE TABLE, ALTER TABLE (ADD/DROP/RENAME COLUMN, RENAME TO), and DROP TABLE in filename-sorted order
- Returns empty map if the schema directory doesn't exist (no error)
- Column types are normalized to uppercase for consistent comparison
- ALTER TABLE ADD COLUMN is idempotent — duplicate columns are skipped
- DROP TABLE removes the table entirely from the schema map
- ALTER TABLE RENAME TO moves all columns to the new table name
- ALTER TABLE RENAME COLUMN preserves all attributes except the name
- CREATE TABLE replaces any prior definition of the same table
- SQL line comments (`--`) are skipped during parsing
- String literals with escaped quotes are handled correctly (no false column detection)
- Supports schema files with extensions: sql, ts, js, mjs, cjs, swift, kt, kts, java, py, rb, go, rs, cs, dart, php
- `parse_spec_schema` supports both inline (`### Schema: table_name`) and multi-table (`### Schema` with `#### table_name`) formats
- Markdown table header rows are skipped during spec schema parsing

## Constraints

- Regex-based SQL parsing only — no SQL parser dependency
- Must handle real-world migration files with mixed DDL and DML statements
- Table-level constraints (PRIMARY KEY, UNIQUE, FOREIGN KEY) are skipped during column parsing

## Out of Scope

- Parsing DML statements (INSERT, UPDATE, DELETE)
- Index or constraint validation
- Supporting non-SQL schema definitions (Prisma, TypeORM decorators, etc.)
- Generating migration files from spec changes
