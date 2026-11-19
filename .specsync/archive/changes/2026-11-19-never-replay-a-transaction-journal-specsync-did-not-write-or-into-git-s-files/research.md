---
change: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
artifact: research
---

# Research

- **Lock and replay.** `acquire_project_lock` (`sdd_version.rs`) calls `recover_pending_transaction` after taking the lock. Every writing command reaches it: `change new`, `answer`, `depend`, `supersede`, `approve`, `start`, `materialize`, `verify`, `reopen`, owner corrections, review, accept, finalize, archive, adopt, retire, send-back, relay repair, doctor's late close and historical reconciliation.
- **Replay.** For each entry the replay runs `safe_project_path`, then `atomic_write_durable` (temp plus rename, keeping the old file's permissions) or `remove_file_durable`.
- **What transactions write**, all through `write_prepared_files_checked`:
  - change records under `.specsync/changes/` and `.specsync/archive/`;
  - the policy, the baseline, the adoption report and the import provenance, all under `.specsync/`;
  - foreign imports under `.specsync/imports/` and in a change's `imported/` directory;
  - during materialization, each affected module's spec and `requirements.md` and its history fragment, resolved by `canonical_module_paths`: the registry first, then `specs_dir/<module>/<module>.spec.md`.
- **Tracking.** `git ls-files --error-unmatch -- <path>` succeeds when the index has the path. `git cat-file -e HEAD:./<path>` succeeds when `HEAD` has it. Both run in the project root, and outside a Git repository both fail, which means not tracked.
- **Versions.** 5.0.0's replay was the same, with plain `fs::write`. The durable envelope arrived in 6.0.0.
