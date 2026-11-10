---
change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
artifact: research
---

# Research: proof that SpecSync 6.0 to 6.2 ignore `.specsync/archive/closed-late/`

**Binaries.** Real release binaries 6.0.0, 6.0.1, 6.1.0, 6.1.1, 6.1.2 and 6.2.0 (sha256 checked), and main
f7180c34.

**Code.** In 6.0.0 and 6.2.0, the roster, the archive preflight, the successor walk and the sequence
accounting read only directly under `.specsync/changes/` (`CHANGES_PATH`) and `.specsync/archive/changes/`
(`ARCHIVE_PATH`). All of `.specsync/archive/` is volatile, so it is not a project input. A top-level
`.specsync/closed-late/` would be a project input, and writing there would stale every open change.

**Synthetic fixture, built with 6.0.0.**

- A: a v1 `accepted` change with drifted input.
- B: a current v1 `accepted` change.
- D: a v1 draft.
- C: a v2 partial finalize in #616's shape.

A and D were moved to `.specsync/archive/closed-late/<date>-<id>/` with a `late-close.json`. Every binary
gave the same results:

- before the move, finalize of C is refused on A;
- after it, audit passes, list omits A and D, and status, ship-status and reopen on A or D say "not found";
- C's finalize resumes and archives, B's v1 archive passes, and audit passes with 0 active;
- every 6.x `change new` can mint A's slug again.

**Site clone at 3d858b83.** All 41 changes were moved. Under 6.2.0, #616 checks, reviews and finalizes with 0
open. 6.0.0 and 6.2.0 audit failed on "acknowledged collision CHG-0032 no longer matches the exact historical
ID set" (`validate_change_sequences` compares each acknowledgement with the enumerated IDs). Dropping that
acknowledgement cleared the collision error. 6.x then asks for an open change covering
`.specsync/change-sequence.json`, a meaningful path. Under 6.0.0, #616's own `change check` overflows a
bounded read whatever doctor does; 6.2.0 does not.

**Existing pieces.**

- `remote_default_ref` resolves the remote default.
- `finalize_change` resumes an `Accepted` v2 record through `validate_finalization_evidence`.
- The archive preflight walks only `ChangeState::Accepted`.
- `migrate 5.0` backfills 5.0.1-era digest fields idempotently.
