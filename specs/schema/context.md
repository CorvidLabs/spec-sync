---
spec: schema.spec.md
---

## Key Decisions

- **Migration replay ordering**: Files are sorted by filename, not modification time. This means migration files should use numeric prefixes (001_, 002_, etc.) for deterministic ordering.
- **Uppercase normalization**: Column types are always uppercased during SQL parsing to allow case-insensitive comparison with spec-documented types.
- **Idempotent ALTER ADD**: If a column already exists in the table, `ALTER TABLE ADD COLUMN` is a no-op. This prevents duplicate columns from repeated migrations.
- **Zero-dependency SQL parsing**: Uses regex-based parsing rather than a full SQL parser. Handles common DDL statements but not every SQL dialect edge case.
- **Two spec schema formats**: Supports both single-table inline (`### Schema: table_name`) and multi-table (`### Schema` with `#### sub-headers`) formats for flexibility in spec authoring.

## Files to Read First

- `src/schema.rs` — All schema parsing logic: SQL DDL replay, spec column extraction, and helper functions.
- `src/validator.rs` — Primary consumer: uses `build_schema` and `parse_spec_schema` for column-level validation.

## Current Status

Fully implemented. Supports CREATE TABLE, ALTER TABLE (ADD/DROP/RENAME COLUMN, RENAME TO), DROP TABLE, and both spec schema formats. Handles string literals, comments, nested parentheses, and table-level constraints.

## Notes

- Virtual tables (`CREATE VIRTUAL TABLE ... USING ...`) are intentionally skipped for column parsing because they use different syntax.
- The `SQL_EXTENSIONS` list covers 16 file types including application code files (ts, py, rb, etc.) that may contain embedded SQL.
