---
change: an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now-and-its-archive-records-which-one
artifact: design
---

# Design

- **Covered for now.** In the accepted-input walk, a changed input that no accepted or archived successor covers is covered for now by the first approved, implementing or verifying change created after the accepted one that declares the input's owning module and a path scope covering it. The walk returns `ProvisionallyCovered` and keeps each `(path, change)`.
- **Reported.** Terminal evidence reads `provisionally-covered` with a reason naming each path and its covering change. `change audit` adds a warning, never an error, so `--strict` exits 0; doctor treats it as covered, as the archive preflight does.
- **Not settled.** A provisionally covered change does not cover another change's input and is not a retirement replacement, because its own currency is only deferred. `change reopen` still accepts it.
- **May archive.** `change archive` and `change finalize` take the change's coverage from the closing check they already run, against the structural roster, and write `coverage.json` (schema version, change id, one `{path, change}` per input covered for now) into the package. The archive's own evidence never reads it.
- **Read back.** `change status` and `change show` on such an archive print each entry with the covering change's present state, and they and `change audit --archives` warn when that change is no longer approved, implementing, verifying, accepted or archived, naming `specsync change reopen <id> --actor <name> --reason <text>`. A warning, not an error.
- **#688.** The stale-input refusal adds one sentence naming each later change whose declared scope covers the input and saying it cannot be archived either; the preflight refusal adds one sentence saying the named change cannot be archived either until it is reopened or a covering change is accepted. Neither replaces the `change reopen` command.

Rejected: downgrading every stale accepted input to a warning, which would hide inputs nobody is working on.
