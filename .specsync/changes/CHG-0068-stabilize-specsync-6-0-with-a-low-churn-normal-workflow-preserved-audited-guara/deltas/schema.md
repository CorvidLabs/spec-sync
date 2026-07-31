## MODIFIED

### REQUIREMENT REQ-schema-001

Schema parsing SHALL replay supported migration DDL deterministically and compare canonical schema
tables without interpreting unsupported DML.

Acceptance Criteria

- Migration files are sorted by filename and CREATE TABLE, ALTER TABLE ADD/DROP/RENAME COLUMN,
  ALTER TABLE RENAME TO, and DROP TABLE replay in exact statement order.
- Missing/unreadable schema inputs, malformed supported DDL, missing referenced objects, canonical
  name collisions, and vacuous configured snapshots return path-aware errors.
- Diagnostics include a one-based line and column plus a concise statement preview.
- Plain CREATE fails on duplicates; IF NOT EXISTS preserves existing state; OR REPLACE explicitly
  replaces it; DROP/RENAME preconditions fail before mutation.
- ANSI, backtick, bracket, mixed-case, quoted-dot, and qualified identifiers share one canonical
  identity parser.
- SQL comments, strings, and quoted content cannot introduce false operations or boundaries.
- Spec schema parsing preserves inline and multi-table Markdown forms and skips header rows.
