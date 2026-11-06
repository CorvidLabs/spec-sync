---
change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
artifact: design
---

# Design

**Command.** `specsync doctor [--fix] [--dry-run] [--close-late <id>|all ...] [--actor <human> --reason <text>] [--json]`

- `src/commands/doctor.rs` renders the command and is added to cmd_change's files.
- `src/change/doctor.rs` holds the scan and the fixes in the change domain.

**Scan.** It is read-only. Each finding records:

- kind and change ID;
- location (active, archived, closed-late);
- reason;
- fix, with its class: safe, decision or report-only;
- the exact command or doctor flag that applies it.

A check that fails to run becomes a finding with its error, for example the #976 overflow on an accepted
change's evidence. The scan continues.

| Kind | Detected by | Fix and class |
|---|---|---|
| merged, never archived | an active package whose `state.json` exists on `remote_default_ref` | decision: `--close-late <id>`; the normal finish is also named when the change is ready |
| accepted, inputs drifted | an active `accepted` change whose accepted-input validation fails, with its reason; an `accepted` package under `archive/changes/` | decision: `--close-late <id>`; report: `change reopen <id> --actor --reason` |
| stopped finalize | active, `accepted`, has `finalization.json` (v2); v1 accepted and not archived | safe: resume `change finalize` (v1: `change archive`) when no other finding blocks it; otherwise it names the blocker |
| stale verification | a verifying or implementing change whose verification is stale for the tree | report-only: `change check <id> --commit` (D2) |
| other-version records | 5.0.1-era ledgers missing digest fields; v1 plans missing the 6.2 sections; a schema newer than the binary; an ID in `changes/` or `archive/changes/` that a closed-late package already holds | safe: `migrate 5.0` for the backfill; report for the rest |

**Fix.**

- `--fix` applies the safe fixes, in a deterministic order.
- `--close-late` applies the decision for the named changes, or for every eligible one with `all` (D3).
- `--close-late` needs `--fix`, `--actor` and `--reason`.
- `--dry-run` with `--fix` prints the same plan and writes nothing.

**Late close (REQ-change-143).**

- **Move.** The package moves, byte for byte, with the durable rename `archive` uses, to
  `.specsync/archive/closed-late/<YYYY-MM-DD>-<id>/`. Its `state.json` keeps its state, and no
  `ChangeState` variant is added.
- **Merge proof.** The package's `state.json`, at its current path, must exist on the remote default. The
  record keeps the commit that added it (`git log -1 --diff-filter=A <ref> -- <state.json>`) and its date.
- **Reads not taken.** Delivery inputs, manifests, digests, verification and the preflight.
- **Collisions (D5).** For a member of an acknowledged sequence collision, the acknowledgement is removed
  from `change-sequence.json` and copied into `late-close.json`. The sequence and the high-water claim are
  untouched.
- **All or nothing.** Every named change is validated first. A write failure restores the packages already
  moved.
- **`late-close.json`.**
  - schema_version, change_id, actor, reason, timestamp
  - from_state, workflow_version, from_path
  - default_ref, default_commit, merged_commit, merged_at
  - verified: false
  - dropped_collision_acknowledgement (optional)

**6.3 reads of the closed-late folder.**

- **Roster and preflight.** Unchanged: they enumerate only `changes/` and `archive/changes/`, so a
  closed-late change is never preflighted, never a successor and never coverage.
- **Lookup.** `load_change` falls back to the closed-late folder only to report it.
  - `change status` and `ship-status` say: closed late on <date> by <actor> (<reason>); merged in <commit>
    on <date>; never re-checked, not verified. A draft adds: never approved.
  - JSON carries `closed_late: {...}` and `verified: false`.
- **Refusals.** reopen, review, check, accept, archive and finalize refuse it with that sentence.
- **New changes.** `change new` refuses its slug.
- **Duplicates.** Doctor reports an ID that a closed-late package and an `archive/changes/` or `changes/`
  package both hold.

**Compatibility test (D4).** `.github/scripts/test-closed-late-compat.sh`:

1. Download the 6.0.0 and 6.2.0 `specsync-linux-x86_64` release assets, pinned by sha256.
2. Build the fixtures with the PR binary and run `doctor --fix --close-late …`.
3. Check the old binaries' audit, list, status, ship-status, reopen, finalize-resume and v1 archive, plus a
   collision fixture.

It runs as a CI job on the Linux lane and as a fledge lane.
