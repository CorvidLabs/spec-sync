## ADDED

### REQUIREMENT REQ-schema-001

Schema parsing SHALL replay supported migration DDL deterministically and compare canonical schema tables without interpreting unsupported DML.

Acceptance Criteria
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
