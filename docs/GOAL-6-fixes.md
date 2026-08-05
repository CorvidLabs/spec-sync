# GOAL: fix the seven findings

**Definition of done:** a fresh project can complete a change end to end, on an ordinary
branch, without editing a file the tool generated — and the drills that assert today's
failures have been inverted to assert the fix.

Findings and reproductions: `docs/6-0-findings.md`.
Every finding has a drill that goes **red when fixed**. That is the signal to invert it.

---

## Order

### 1. The ledger design problem — findings 2, 4, 6

One cause, three symptoms. Highest leverage.

`.specsync/change-sequence.json` is committed, branch-sensitive state that references change
directories which are also branch-sensitive, with nothing keeping the two in step.

| Finding | Symptom |
|---|---|
| 6 | `change new` fails outright on a branch missing an earlier change's directory |
| 4 | the review gate rejects a commit containing the ledger |
| 2 | `audit --strict` reports the ledger as uncovered right after `change new` |

Finding 6 is the one to fix first — it blocks the first command in the workflow.

Candidate directions, none yet evaluated:

- Ledger entries tolerate a missing directory: treat as not-active-*here* rather than an error
- Keep the ledger out of version control, deriving the sequence from directories present
- Reconcile ledger against directories on load, warning rather than failing

Decide the direction before writing code. The first is narrowest; the second removes the
whole class but changes what the ledger is for; the third is a half-measure worth rejecting
explicitly rather than drifting into.

**Drill:** `031-specsync-merge-conflict.sh`

### 2. Scaffold versus contract gate — finding 7

`specsync new` emits placeholder prose; effective-contract validation promotes that stub
warning to a hard error at finalize. The tool authored the text it then rejects.

Two fixes, not exclusive:

- **At the gate:** scope any stub exemption to sections *this change did not author* — the
  applied `SpecSection` delta keys name exactly which ones it did — plus the
  `canonical_applied && Verifying` case where no delta is replayed. Route it through
  `IgnoreRules` so `.specsyncignore` works here and every suppression is reported.
- **At the source:** have `specsync new` emit scaffolds that pass the gate they will be
  judged by.

**Do not** filter `StubSection` warnings wholesale. That was tried and reverted: a
`## MODIFIED` delta with an empty body writes an emptied section into the canonical spec, and
the stub warning is the only gate catching it. See finding 7's "Bearing on decision A".

**Drill:** `032-next-action-loop.sh`

### 3. `reopen` from archived — finding 5

Decided. Scope is written into finding 5: un-archive then reopen, because `reopen_change`
writes to the *active* directory while an archived change lives under `archive/changes/`.
Relaxing `require_state` alone would create two directories claiming one ID.

Open sub-question to settle first: should un-archiving be a **distinct audited event**?

**Drill:** `030-correct-owner-6.sh`

### 4. Already fixed

Finding 1 — `init` writes usable verification commands, or warns. `leif/finalize-adopt`
`3c28e29`, suite green, pushed.

---

## Standing rules

- **The drills are the judge.** The Rust suite is green while all seven of these exist; it
  cannot see this class of defect (finding 3).
- **Invert the drill in the same change as the fix.** A finding whose drill still asserts the
  old behaviour is not finished.
- **Do not swallow errors.** Two false leads today came from `>/dev/null 2>&1` turning a
  fixture mistake into an apparent product bug.
- **Assert observed wording, never guessed wording.**
- Warn before any edit that invalidates an approval or verification digest.
- `src/change.rs` is a strict-validation path: each fix needs a change workspace and a human
  approval digest.

## Elevate to human

- The ledger direction (§1) — three candidates, materially different consequences
- Whether un-archiving is a distinct audited event (§3)
- Anything touching digest, acceptance, or archive-integrity semantics
- Any approval digest, scoped review, or admin merge
