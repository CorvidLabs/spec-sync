---
change: CHG-0100-fail-closed-in-text-lifecycle-views-when-a-correction-ledger-is-invalid
artifact: tasks
---

# Tasks

- [x] Add a non-disclosing correction-ledger health query for text renderers.
- [x] Make text `show`, `status <id>`, and aggregate `status` fail closed when that query is invalid.
- [x] Add domain and command-module regression tests with malformed `corrections.json`.
- [x] Update the sandbox #17 drill and issue disposition after the product behavior changes.
- [x] Run scoped verification and the sandbox regression drill.
