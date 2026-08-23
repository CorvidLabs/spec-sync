---
id: db-tables-without-schema-dir-must-not-gate-a-release-on-advice-the-reader-cannot-take
state: implementing
type: bug_fix
base_commit: f94ff7e473ef6300180b9d79c1be3fe48c9ab527
---

# Db_tables without schema_dir must not gate a release on advice the reader cannot take

## Intent

db_tables without schema_dir must not gate a release on advice the reader cannot take

## Affected Canonical Specs

- `validator`

## Acceptance Criteria

- A spec declaring db_tables in a project with no schema_dir passes 'specsync check --strict' with exit 0, while the 'DB table validation skipped' disclosure remains visible as a notice. Where schema_dir IS configured, a declared table absent from the readable schema still errors, unchanged. The discriminating test fails on a binary built from a separate checkout at f94ff7e4 and its vacuity control passes on both.

## No-spec Rationale

behaviour within the validator contract: an unactionable disclosure moves from warnings to notices; no spec text changes
