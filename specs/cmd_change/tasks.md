---
spec: cmd_change.spec.md
---

# Tasks

- [x] Fail `change status` when `.specsync/sdd.json` cannot be parsed (REQ-cmd-change-021).
- [x] Commit the archive move from `change finalize` (REQ-cmd-change-022).
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
- [x] Treat a change directory that holds a file but no state.json as still open, and skip an empty husk.
- [x] Make text `Next:` name re-approval for every approved, implementing or verifying change whose approval is not current, through `change::approval_current_for_guidance` (REQ-cmd-change-005, REQ-change-111).
- [x] Disclose a non-empty `verification_ignored_paths` list in `change status` and `change check` text (with the standing note) and in `summary.verification_ignored_paths`, name an invalid entry as the text `Next:` and as a `change ship-status` blocker instead of stale verification (REQ-cmd-change-024, issue #825).
- [x] Archive-first CI: query HEAD check-runs first in ship-status and rewrite trust guidance and stage copy for verify-locally → review → ship → push the archive tip (REQ-cmd-change-007, REQ-cmd-change-019)
- [x] Render `approved_deltas` verbatim in single-record change JSON (`status`, `show`, `new`, `answer`, `depend`, `supersede`), from the in-lock mutation snapshot for mutations (REQ-cmd-change-023).
- [x] Pass `change audit --archives` to `change::audit_project_with_archives`, which also walks every archive's workflow-version history (REQ-cmd-change-027, #856).
- [x] Name only a state-admitted verb in the ship-status product stage: approve for a draft, reopen for stale accepted or archived evidence, check otherwise (#690, REQ-cmd-change-033).
