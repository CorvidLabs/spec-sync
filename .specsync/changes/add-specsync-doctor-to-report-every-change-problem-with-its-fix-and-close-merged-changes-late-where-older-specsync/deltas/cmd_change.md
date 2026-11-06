## ADDED

### REQUIREMENT REQ-cmd-change-035

`specsync doctor` SHALL report every problem it finds in the repository's change records, each with its fix classed as safe, decision or report-only; with `--fix` it SHALL apply only the safe fixes, and it SHALL apply a late close (REQ-change-143) only to the changes named with `--close-late`, and only with a human actor and a reason.

Acceptance Criteria
- Findings cover: a change merged but never archived; an accepted change whose inputs drifted, including an `accepted` package under `.specsync/archive/changes/`; a finalize that stopped halfway, including a workflow-v1 change accepted but not archived; stale verification; and records another SpecSync version wrote (missing 5.0.1-era digest fields, workflow-v1 plans without the 6.2 sections, a newer schema, an ID that a closed-late package already holds).
- Each finding names its kind, change, reason, fix class and the exact command or doctor flag that applies the fix. Text and JSON carry the same findings.
- A check that cannot run becomes a finding that names its error, and every other finding is still reported.
- Plain `doctor` writes nothing. `--fix --dry-run` reports exactly what `--fix` would do with the same arguments and writes nothing.
- The safe fixes are the `migrate 5.0` backfill, and resuming a stopped finalize (workflow v1: `change archive`) when no other finding blocks it. Stale verification is report-only and names `change check <id> --commit`.
- `--close-late <id>` (repeatable) or `--close-late all` closes the named or every eligible change late, only with a non-empty `--actor` and `--reason`. Every named change is checked before anything is written, and one refusal writes nothing and names the refused change.
- A late close of a collision member says that the ledger edit needs an open change declaring `.specsync/change-sequence.json` wherever meaningful files require one.
