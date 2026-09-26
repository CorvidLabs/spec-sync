---
spec: cmd_change.spec.md
---

# Tasks

- [x] Add all lifecycle commands
- [x] Add structured JSON projections
- [x] Add consistent error handling
- [x] Complete release validation
- [x] Dispatch audited stale-accepted reopen in text and JSON formats
- [x] Dispatch accepted metadata correction with equivalent text and JSON projections
- [x] Dispatch exact acceptance-owner correction with equivalent text and JSON projections
- [x] Dispatch transactional batch correct-owner selection (paths/manifest/all-missing)
- [x] Dispatch explicit pass/block scoped-review verdicts with equivalent text and JSON
- [x] Delegate answer, depend, and supersede to locked domain ledger validation before persistence
- [x] Render successful mutations from their validated domain transaction snapshots
- [x] Select the locked normal/strict summary instead of recomputing machine output after persistence

- [x] Use recorded verification content currency consistently in the product stage and readiness (#745).
- [x] Add preserved-content squash and stale-content ancestor regression controls.
- [x] Stage lifecycle commits by explicit literal pathspecs (tracked edits plus the change's owned untracked paths), never `git add -A`, and list every other untracked file on stderr (REQ-cmd-change-017).
- [x] State in `run_checked_commit`'s doc comment what happens when the second verification fails, and name the materialize commit in that error.
