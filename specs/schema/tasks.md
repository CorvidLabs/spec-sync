---
spec: schema.spec.md
---

## Tasks

- [ ] Support CREATE INDEX tracking for schema completeness
- [ ] Add VIRTUAL TABLE column extraction (FTS5 columns)
- [ ] Support multi-statement migration files with transaction wrappers (BEGIN/COMMIT)

## Done

- [x] CREATE TABLE parsing with column constraints (NOT NULL, DEFAULT, PRIMARY KEY)
- [x] ALTER TABLE ADD COLUMN with idempotent duplicate handling
- [x] DROP TABLE / DROP COLUMN support
- [x] ALTER TABLE RENAME TO / RENAME COLUMN support
- [x] Spec schema extraction (inline and multi-table formats)
- [x] String literal and comment handling in paren matching
- [x] Table-level constraint skipping (PRIMARY KEY, UNIQUE, CHECK, FOREIGN KEY, CONSTRAINT)
- [x] SQL keyword filtering for column names

## Gaps

- No support for VIRTUAL TABLE column extraction
- No transaction wrapper handling (BEGIN/COMMIT blocks)
- No CREATE INDEX tracking

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
