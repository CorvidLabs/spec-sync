---
change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
artifact: research
---

# Research

## What the approval checks, in order

Read on main 3bebe8b0, `approve_definition_with_relay`:

1. The relayed decision: `--words`, `--where` and `--recorded-by` come together, need `--actor`, and
   the relay names that actor as the one who decided.
2. The lifecycle lock. Acquiring it refuses a lock path with a symlink component, creates
   `.specsync/change.lock`, and recovers an interrupted write from `.specsync/change-transaction.json`.
3. `load_change`.
4. `ensure_no_sequence_collision`.
5. The state guard, with the one exception for an accepted change whose definition approval is stale.
6. `list_changes_checked`.
7. `bind_legacy_archive_baseline_authority`, which changes the record in memory.
8. `validate_definition`.
9. `validate_delta_files`.
10. `validate_declared_path_ownership`.
11. `ensure_change_clear_of_verification_ignored`.
12. For a relayed decision on a workflow-v2 change: raise the workflow version and save `state.json`.
13. Build the approval record: the definition digest, the approvals ledger, the approved scope, the delta
    body digests and the delta bases (#843), then the actor. `--portable-5-0-1` builds its pair through its
    own function with the same parts.
14. Write `approvals.json`, `state.json` and `change.md`.

Each of 3 to 11 and each part of 13 returns its first problem as an `Err(String)`. None of them writes.

## What a refused approve writes

Measured on a throwaway project with the 6.3.0 code: a refused `change approve` leaves every file as it
was except `.specsync/change.lock`, which it creates when absent. The Git index is unchanged. The hash
cache, `.specsync/hashes.json`, is written by `specsync check`, not by approve. The read cache lives under
the Git directory and only the read-only commands' read scope writes it.

## Options considered

- A separate command, such as `change check-approve`. Not chosen: a flag on `change approve` keeps the
  flags the two take identical by construction.
- Re-running the checks in the command layer. Not chosen: that is the copy of the rules #1091 exists to
  remove, one level down.
- Running the real approval and putting the files back. Not chosen: it writes, and an interruption would
  leave an approval nobody gave.
- Listing every problem inside each check. Not in this release, by Leif's choice.
