---
change: doctor-fix-refuses-under-an-inherited-verification-context-and-read-only-doctor-still-reports
artifact: context
---

# Context

Issue #997, a follow-up from #994 (`specsync doctor`, #833). The root dispatcher refuses a
`check`, `change` or `lifecycle` command under an inherited `SPECSYNC_VERIFICATION_CONTEXT`
(REQ-cli-003, `reject_recursive_lifecycle_dispatch` in `src/main.rs`), but `doctor` is not in
that list, though `doctor --fix` writes lifecycle state: it runs `change finalize` and
`change archive` as safe fixes, the `migrate 5.0` backfill, and late closes that move packages
and edit `.specsync/change-sequence.json`. It was left out of #994 because it changes the cli
contract.

Leif decided the behaviour on 2026-11-06 (orc relay, recorded on #997): read-only
`specsync doctor` still reports under an inherited verification context; `--fix` and
`--close-late` refuse there like every other lifecycle writer. `--close-late` requires
`--fix`, so the guard keys on `--fix`. Leif then selected "#997 doctor recursion guard" to build
on 2026-11-13T03:28Z.
