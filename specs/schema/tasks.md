---
spec: schema.spec.md
---

## Tasks

## Post-5.0 Roadmap

- [ ] Support CREATE INDEX tracking for schema completeness
- [ ] Add VIRTUAL TABLE column extraction (FTS5 columns)

## Done

- [x] CREATE TABLE parsing with column constraints (NOT NULL, DEFAULT, PRIMARY KEY)
- [x] ALTER TABLE ADD COLUMN with idempotent duplicate handling
- [x] DROP TABLE / DROP COLUMN support
- [x] ALTER TABLE RENAME TO / RENAME COLUMN support
- [x] Spec schema extraction (inline and multi-table formats)
- [x] String literal and comment handling in paren matching
- [x] Table-level constraint skipping (PRIMARY KEY, UNIQUE, CHECK, FOREIGN KEY, CONSTRAINT)
- [x] SQL keyword filtering for column names
- [x] Replay supported DDL in exact statement order within filename-sorted migrations
- [x] Return path/line/column diagnostics for missing directories, unreadable files, malformed DDL, missing objects, and collisions
- [x] Canonicalize ANSI/backtick/bracket quoted, mixed-case, and qualified table identities
- [x] Preserve snapshot state on duplicate CREATE and RENAME-target collisions
- [x] Support multi-statement migration files and ignore transaction-control/DML statements

## Gaps

- No support for VIRTUAL TABLE column extraction
- No CREATE INDEX tracking
- Validator/CLI consumers must be restacked in #453 to use the fallible snapshot directly and remove the compatibility wrappers

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
