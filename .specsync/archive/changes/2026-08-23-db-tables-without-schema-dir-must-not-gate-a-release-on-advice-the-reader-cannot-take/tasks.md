---
change: db-tables-without-schema-dir-must-not-gate-a-release-on-advice-the-reader-cannot-take
artifact: tasks
---

# Tasks

- [x] Establish that #672's fix cannot reach this path (it lives inside `schema_dir.is_some()`)
- [x] Reproduce the adopter's exact shape: `db_tables` declared, no `schema_dir`, no `.sql` files
- [x] Find the existing notices channel and confirm notices cannot gate `compute_exit_code`
- [x] Find the precedent (`Planned source mapping`) rather than inventing a convention
- [x] Move the disclosure from warnings to notices; keep the fix suggestion
- [x] Verify strict now exits 0 AND the message is still printed
- [x] Controls: missing table still errors when the schema is readable; configured+present stays clean
- [x] Discriminator red on a separate checkout at `f94ff7e4`; vacuity control green on both
- [x] Full suite, fmt, `check --strict`
