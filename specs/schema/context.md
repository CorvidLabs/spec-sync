---
spec: schema.spec.md
---

## Key Decisions

- **Migration replay ordering**: Files are sorted by filename, not modification time. This means migration files should use numeric prefixes (001_, 002_, etc.) for deterministic ordering.
- **One fallible snapshot**: `build_schema_snapshot` owns directory loading, UTF-8 reads, canonical identity, and DDL replay. Validation consumers must use its `Result` instead of combining an empty map with a second best-effort error scan.
- **Exact statement ordering**: Supported DDL is discovered outside SQL comments and replayed in byte order within each file. String literals and quoted identifiers do not create false operation boundaries.
- **Collision semantics**: Plain duplicate CREATE, missing DROP/RENAME sources, and colliding RENAME targets fail without mutating the existing snapshot. IF EXISTS, IF NOT EXISTS, and OR REPLACE are the only explicit leniency/replacement paths.
- **Canonical table identity**: ANSI double quotes, backticks, SQL Server brackets, case differences, whitespace around qualifiers, and qualified names pass through `canonicalize_table_name` for every CREATE/ALTER/DROP transition. Canonical rendering preserves dots inside quoted segments; `canonical_table_leaf` performs safe unqualified matching.
- **Uppercase normalization**: Column types are always uppercased during SQL parsing to allow case-insensitive comparison with spec-documented types.
- **Idempotent ALTER ADD**: If a column already exists in the table, `ALTER TABLE ADD COLUMN` is a no-op. This prevents duplicate columns from repeated migrations.
- **Zero-dependency SQL parsing**: Uses regex-based parsing rather than a full SQL parser. Handles common DDL statements but not every SQL dialect edge case.
- **Two spec schema formats**: Supports both single-table inline (`### Schema: table_name`) and multi-table (`### Schema` with `#### sub-headers`) formats for flexibility in spec authoring.

## Files to Read First

- `src/schema.rs` — All schema parsing logic: SQL DDL replay, spec column extraction, and helper functions.
- `src/validator.rs` — Primary consumer: uses `build_schema` and `parse_spec_schema` for column-level validation.

## Current Status

The schema-owned #435 slice is implemented: one ordered fallible snapshot covers CREATE/ALTER/RENAME/DROP, malformed input, missing objects, quoted/qualified identities, deterministic collisions, and positioned diagnostics. The validator/command integration remains a dependency of the #453 restack so empty snapshots are never treated as “validation disabled.”

## Notes

- Virtual tables (`CREATE VIRTUAL TABLE ... USING ...`) are represented for table existence with an empty column set because their module-specific column syntax is not parsed.
- The `SQL_EXTENSIONS` list covers 16 file types including application code files (ts, py, rb, etc.) that may contain embedded SQL.
- `build_schema` and `build_schema_with_retired` remain lossy compatibility wrappers only so this isolated schema commit compiles before #453 adopts `SchemaSnapshot`; validation must use `build_schema_snapshot` or surface `schema_read_errors`.
