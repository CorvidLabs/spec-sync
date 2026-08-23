---
change: a-schema-that-cannot-be-parsed-must-not-report-every-table-as-missing
artifact: tasks
---

# Tasks

- [x] Reproduce with two plain .sql files rather than trusting the report
- [x] Add an unrelated table to the fixture to measure the cascade's blast radius
- [x] Find the cascade: an error swallowed into an empty set that reads as absence
- [x] Separate "unknown" from "absent" without moving four public signatures
- [x] Resolve availability lazily, on the error path only
- [x] Narrow the duplicate-column check to agreeing redeclarations, keeping type conflicts fatal
- [x] Discriminator proven red on a separate checkout
- [x] TRUE vacuity control proven green on BOTH binaries
- [x] Three live controls on the reproduction: missing still missing, type conflict still fatal,
      unparseable schema reports only its own error
- [x] Full suite, fmt, `specsync check`
