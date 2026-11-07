---
change: report-production-source-no-declared-module-owns-before-approval-and-review-and-never-call-such-a-change-ready-to
artifact: context
---

# Context

Issues #986 (found shipping #980) and #956 (found shipping #900's covering change), both the same gap. A `--no-spec-change` change with no declared specs whose scope delivers production source that a module owns passes `change approve`, `check --commit` and `review`. Then `ship-status` and `ship --dry-run` say "ready to finalize", and only `change ship` refuses with "acceptance input `<path>` is production source without deterministic canonical ownership". The refusal names no recovery. The recovery is a send-back, declaring the owning module, re-approval, re-verification and a fresh review, so one human approve-and-review cycle is wasted every time.

Root cause:
- `validate_declared_path_ownership` returns early for no-spec-change changes with no declared specs, as REQ-change-053 says ("finalize still enforces").
- Approve cannot see which owned files a directory scope (for example `src/billing/`) will deliver.
- Ship preflight and ship-status never run finalize's ownership resolution (`acceptance_input_owners`).

Intent: SHIP-4.a ("The answer names the stage I am on and the one thing holding it") and CHANGE-10 ("When `change approve` refuses an affected path that no module owns, the error says which spec to edit").

Out of scope: who owns what (REQ-change-053's resolution rules are unchanged), owner corrections for reopened changes, and paths that do not exist yet at approve, which are still enforced at finalize and now also at ship preflight.
