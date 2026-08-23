---
change: a-schema-that-cannot-be-parsed-must-not-report-every-table-as-missing
artifact: context
---

# Context

Reported as #672 from adopting 6.0.0-rc.2 on a real repository. It is the defect that makes
`strict` unusable there, and it is two defects sharing one cause.

## Reproduced with two plain .sql files

    schema/001_init.sql   CREATE TABLE events (id TEXT PRIMARY KEY, url TEXT NOT NULL DEFAULT '');
                          CREATE TABLE visitors (id TEXT PRIMARY KEY);
    schema/002_backfill.sql  ALTER TABLE events ADD COLUMN url TEXT NOT NULL DEFAULT '';

Before:

    ✗ DB table not found in schema: events      <- created correctly in 001_init.sql
    ✗ DB table not found in schema: visitors    <- NOTHING to do with the ALTER
    ✗ 002_backfill.sql:2:1: ALTER TABLE ADD duplicates existing column `events.url`
    0 passed, 1 failed

`visitors` was added to the fixture specifically to measure blast radius. It is collateral: a
table with no relationship to the duplicate `ALTER` reports missing.

The suggested fix compounded it — "add a CREATE TABLE migration" for tables whose `CREATE TABLE`
is present and correct.

## Defect 1: the cascade is an error swallowed into a value that means something else

`get_schema_table_names` (`src/validator.rs:79`):

    let snapshot = match schema::build_schema_snapshot(&schema_dir) {
        Ok(snapshot) => snapshot,
        Err(_) => return HashSet::new(),          // <- here
    };
    schema_table_names_from_snapshot(&snapshot, config).unwrap_or_default()   // <- and here

Two sites, one line apart, both converting "I could not determine the tables" into "there are no
tables". The caller then reads an empty set as proof of absence and reports every declared table
as missing.

This is the recurring shape of this release, inverted: a category is empty for want of INPUT, and
the code reads that as a VERDICT. Note `Err(_)` — the error is not even inspected.

## Defect 2: the parser understands an intent SQLite cannot express

`replay_sql`'s `AddColumn` arm already accepted a duplicate when `if_not_exists` was set. But
**SQLite has no `ADD COLUMN IF NOT EXISTS`**, so a SQLite author cannot reach that branch.

The pattern is not sloppiness, it is the only correct one available:

- `CREATE TABLE` carries the column, so fresh databases are right immediately
- a later bare `ALTER TABLE ADD COLUMN` back-fills older ones, executed with the error discarded
  (in Go, literally `_, _ = db.Exec(stmt)`)

Remove either half and one of the two database populations is wrong.

## Why the fix narrows rather than removes

Accepting every duplicate would make a genuine mistake silent. `SchemaColumn` carries
`col_type`, so the two cases are distinguishable:

- redeclaration that AGREES with the existing column — the idempotent back-fill — is a no-op
- redeclaration that CONTRADICTS it — a real defect — still fails, now naming both types

The check keeps its value exactly where it had any.
