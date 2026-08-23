---
id: a-schema-that-cannot-be-parsed-must-not-report-every-table-as-missing
state: archived
type: bug_fix
base_commit: 1d474ed905f8f1155f50488ba1f4906311de8940
---

# A schema that cannot be parsed must not report every table as missing

## Intent

a schema that cannot be parsed must not report every table as missing

## Affected Canonical Specs

- `schema`
- `validator`

## Acceptance Criteria

- The two-migration SQLite idempotent back-fill reported in #672 validates cleanly: a CREATE TABLE carrying a column plus a later bare ALTER TABLE ADD COLUMN goes from three errors to one spec passing. An unparseable schema reports only its own parse error and no longer claims every declared table is absent, including tables created correctly in unrelated files. A genuinely missing table still reports missing, and an ALTER that redeclares a column with a CONTRADICTING type still fails on any binary.

## No-spec Rationale

behaviour within existing module contracts: an unreadable schema degrades to unknown instead of absent, and an agreeing ADD COLUMN redeclaration is a no-op; no spec text changes
