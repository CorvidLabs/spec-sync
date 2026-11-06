---
change: close-a-change-that-merged-but-was-never-archived-in-one-audited-late-close-that-records-it-as-never-re-checked
artifact: design
---

# Design

**Command.** `specsync change close-late <id>... --actor <human> --reason <text> [--dry-run]`, dispatched
by the change command handler (cmd_change) with its arguments in cli_args.

**State.** `ChangeState::ClosedLate`, serialized `closed_late`. It is a separate terminal state, so every
exhaustive match decides it explicitly. No path that trusts `Archived` or `Accepted` evidence can take it
for either: the archive preflight walks only `Accepted`, the successor walk refuses anything that is not
`Accepted` or `Archived`, and `require_state` in reopen, review and finalize does not list it. Older
SpecSync versions fail to parse it and fail closed (D1).

**Merge proof.** `remote_default_ref` (the existing helper: `origin/HEAD`, else `origin/main` or
`origin/master`). The package's `state.json` must exist at its current path in that tree. The commit that
added it (`git log -1 --diff-filter=A <ref> -- <state.json>`) and its committer date go into the record. No
network call is made, and nothing is fetched.

**Record.** `late-close.json` in the package:

- schema_version, change_id, actor, reason, timestamp
- from_state, workflow_version
- default_ref, default_commit, merged_commit, merged_at
- verified: false

Every other file in the package keeps its bytes. That includes a half-written `finalization.json`,
approvals and verification records, which stay as history and are never read as evidence of a close.

**Write.**

1. The project lock is held.
2. Every ID is validated before anything is written.
3. Each active package moves into `.specsync/archive/changes/<today>-<id>/` with the durable rename that
   archive uses. A package already in the archive stays where it is.
4. `state.json` and `late-close.json` are written durably.
5. If a write fails, the packages already handled are restored, and the error says what was restored.

`--dry-run` stops after validation and prints the plan.

**Reads that are not taken.** Delivery inputs, acceptance manifests, the acceptance input digest,
verification and the archive preflight. That is the point of SHIP-12.c, and it is why CHG-0026 can close.

**Reporting.**

- `change status` text: `closed late on <date> by <actor> (<reason>); merged in <commit> on <date>; never
  re-checked, not verified`. A change closed from draft adds `never approved`.
- JSON: `state: closed_late`, `late_close: {...}`, `verified: false`.
- ship-status names the late close, gives no next action, and does not call the change archived.
- `change audit` counts it closed.
