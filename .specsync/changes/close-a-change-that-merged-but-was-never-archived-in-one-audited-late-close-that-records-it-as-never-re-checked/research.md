---
change: close-a-change-that-merged-but-was-never-archived-in-one-audited-late-close-that-records-it-as-never-re-checked
artifact: research
---

# Research

- **Archive preflight.** It sits in `archive_change_with_options`
  (src/change/historical_definition_digest_matches.rs) and walks only `ChangeState::Accepted` records
  through `validate_accepted_inputs_recursive`. A `closed_late` record drops out without a new skip.
- **Successor walk.** `validate_accepted_inputs_recursive` refuses a successor that is neither `Accepted` nor
  `Archived`, so a closed-late change can never be coverage.
- **Partial-finalize resume.** `finalize_change` resumes an `Accepted` workflow-v2 record through
  `validate_finalization_evidence`, then archives it. That is #616's path once its blocker is gone.
- **Legacy changes.** `finalize_change` refuses workflow v1 and names `change accept` + `change archive`.
  close-late takes neither.
- **Merge proof.** `remote_default_ref` (src/change/validate_loaded_change.rs) and `upstream_archive_of`
  (src/change/consider_accepted_evidence_anchor.rs) already resolve the remote default and the commit that
  added a package.
- **Reopen precedent.** `reopen_change` writes an audited `ReopenRecord` with actor, reason and from_state.
  Its `LegacyAcceptanceUnreconstructible` cause records evidence that cannot be rebuilt (#752).
- **Site probe.** On a throwaway clone of site origin/main 3d858b83, reopen of CHG-0026 fails in 1.8 s
  with "`git cat-file --batch` output exceeds deterministic bounds" (#976).
