---
change: an-orphaned-verification-commit-must-be-reopenable
artifact: research
---

# Research

## Parallel implementations — searched, not assumed

Whole-file greps across `src/`, no range reads:

| symbol | hits | verdict |
|---|---|---|
| `stale_/current_acceptance_input_digest` | 25 | all classified below |
| `authenticate_accepted_evidence` | 5 non-test callers | all preserved via wrapper |
| `verification_commit_is_accepted_current` | 13 | non-test sites are :13590, :13595, :14314-14316 |
| `reopen_change` | 1 non-test caller | `src/commands/change.rs:205` |
| `reopenings` | 30 | none besides :1979 read the digest fields |

**The live trap — `reopened_change_preserves_sequence_history` (`:1979`).** A second, independent
encoding of "a reopen implies the digests differ". Patched to read the recorded cause.

**`backfill_change_ledger` (`:3314`)** — same idiom, third instance. NOT broken: it is the
`migrate 5.0` path and short-circuits at `:3286` for records carrying both digests. Identical
digests remain unrepairable there for a good reason — a backfilled record has no recorded cause, so
drift genuinely cannot be proven.

**`staged_accepted_snapshot_is_closing_authenticated` (`:13590`/`:13595`)** — uses
`verification_commit_is_accepted_current` WITHOUT the two fallbacks. This looks like a fourth copy
and is not: it is the write-path working-tree fallback, deliberately stricter because its caller is
minting a trusted transition rather than reading one. Recorded here so nobody "unifies" it with the
new predicate and quietly widens #660's laundering guard.

**`validate_archived_integrity_inner` (`:14415-14477`)** — found by the adversarial pass, missed by
the first survey. A complete parallel re-implementation of accepted-evidence authentication for
ARCHIVED records, containing NO anchor check. Consequence, measured: `check` reports an archived
unanchored package as `AuthenticatedHistory` while `ensure_closing_approval_valid` errors. Left
alone deliberately — archived records are inert history and are routinely unanchored on a
squash-merged main, so adding the check would turn essentially every existing archive red.

## What the adversarial pass corrected

- The first design claimed invariant 18 "stays true as written". It does not — invariant 18
  constrains audited reopen by name, and this change is exactly what it forbade. Amended.
- The ":1979 project-wide freeze" framing was overstated: the validator returns false for any
  `from_state != Accepted`, and 6.0's finalize accepts and archives atomically, so it bites only
  workflow-v1 Accepted-origin reopens. Patch retained; framing corrected.
