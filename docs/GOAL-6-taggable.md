# GOAL: a taggable spec-sync 6.0.0

**Definition of done:** a clean `v6.0.0` tag where local `specsync` catches essentially every
"spec-sync reason" that currently turns CI red, and no agent is forced into a ship dance.

**Not done until:** the failure class below is empty on a real PR, twice in a row, without a human
knowing a workaround.

---

## The failure class we are eliminating

CI red for *spec-sync reasons*, not ordinary test failures:

| Failure | Status |
|---|---|
| Stale verification evidence | **root cause confirmed** — see §1 |
| Incomplete change artifacts / missing requirement evidence | fixed in #500 (fail-early + named artifact) |
| Approval digest invalidation after edits | open — no warning before the edit |
| Ship / finalize tip thrash | removed by #499; finalization now has *no* trigger (§2) |
| TODO/placeholder artifacts passing approval | open — #495 |

---

## §1 — Stale verification evidence (highest value)

**Confirmed mechanism.** `project_input_digest` derives content from the **Git index**
(`inspect_git_candidates` → `git ls-files --stage`), but `change check` materializes the delta into
the **working tree**. Verification runs the suite against the working tree and records a digest of
index content that was never tested. Staging that same delta then invalidates the evidence it just
produced. Non-git projects are immune (`capture_non_git_candidates` reads the filesystem), which is
the tell that the index path is the anomaly.

**Fix, staged:** a fail-early guard in `verify_change_with_strict` comparing working tree vs index
for the canonical specs a delta materializes, refusing before the suite runs with a message naming
the file, the action, and the reason. Touches no digest semantics.

**ANSWERED.** The index-based digest is *intended*. Verifying while the materialized spec is
unstaged is a supported flow — spec-sync's own fixtures depend on it, which is why a fail-early
"stage it first" guard broke `stale_accepted_change_*` and two others. That guard was wrong and is
not in the tree.

The real defect is narrower: **verify → stage → stale**, with no warning. Deterministic workaround
that provably terminates: `change check` → commit the materialized spec → `change check` → commit
evidence. Confirmed on CHG-0079/CHG-0080 — `audit --strict` passed *after* the evidence commit,
because `.specsync/changes/` is excluded from the project-input digest.

### §1 fix — analysis complete, NOT implemented

Call sites (`src/change.rs`):

| Caller | `allow_verified_tree_adoption` | `require_scoped_review` |
|---|---|---|
| `accept_change` (5315) | `false` | `false` |
| `finalize_change` (5540) | **`true`** | `true` |

Finalize already adopts a changed **commit** (5343-5345). It never gets there because
`validate_verification_for_commit_binding` (5131) refuses first, at 5148:
`verification.workspace_digest != project_input_digest(root)?`.

**Two designs, and they are not equivalent:**

1. *Adopt the digest* alongside the commit. Small. **But it weakens evidence** — it declares "the
   tree changed since we tested, accept anyway." The digest exists precisely to prove what was
   tested. Do not do this without deciding that trade deliberately.
2. *Re-record at finalize* — finalize re-runs verification against the committed tree. Evidence stays
   honest because it was genuinely tested. Costs a suite run at finalize, but replaces the manual
   second pass, so it is time-neutral and requires no author knowledge. **Recommended.**

Design 2 is what "re-record at finalize" meant and is the one to build. It also composes with §2:
if finalize is triggered post-merge, the re-record happens there automatically.

### Implementation hazard — DEADLOCK. Read before writing code.

`accept_change_with_gate` (5318) opens with `let _lock = acquire_project_lock(root)?;`.
`verify_change` (2351) **also** calls `acquire_project_lock`, which is a `file.lock_exclusive()`
flock (405-433). Calling `verify_change` from inside acceptance therefore blocks forever — it hangs
rather than failing, which is worse than any error.

The re-record must be built as:

1. Extract the body of `verify_change_with_strict` below its `acquire_project_lock` into a
   lock-free `verify_change_locked(root, id, strict)`.
2. `verify_change_with_strict` becomes: acquire lock, call the inner.
3. `accept_change_with_gate` calls the **inner** while holding its own lock, when
   `allow_verified_tree_adoption` is set and `validate_verification_for_commit_binding` failed.
4. Re-load verification and re-validate. Fail if it still does not bind.

Do not shortcut by dropping and re-taking the lock — that opens a window where another process can
mutate the workspace between verification and acceptance, which is the exact race the lock exists to
prevent.

Validate against `stale_accepted_change_*`, `verification_freshness_*`, and any test asserting that
acceptance does not re-run commands. Expect the finalize path to get materially slower — that is the
intended trade: one automatic suite run replaces one manual pass the author had to know about.

## §2 — Finalization has no trigger — BLOCKED ON §1, and they are one task

Post-merge finalize **cannot work as a standalone workflow.** Confirmed on main at `4c62169`:

```
$ specsync change finalize CHG-0079-...
error: cannot accept stale verification: verification commit is not an ancestor of HEAD

$ git merge-base --is-ancestor 14abe71 origin/main   ->  NOT an ancestor
$ git log -1 --format="%H %p" origin/main            ->  one parent = squash
```

**Squash merge destroys the verification commit binding.** Evidence records the branch tip; the
squash creates a new commit with no branch ancestry, so the recorded commit is not reachable from
main. Every squash-merged change is permanently unfinalizable by commit binding alone.

This is not the scoped-review requirement (that comes later, and is a separate gate). It fires first.

**Therefore:** a post-merge finalize job is only viable if finalize *re-records* verification against
the merged tree — §1 design 2. Re-running on main sets `commit = HEAD` and refreshes the digest, so
ancestry and staleness both resolve by construction.

Options if re-record is rejected: switch main to merge commits instead of squash (preserves
ancestry, changes history shape), or accept that changes are finalized pre-merge.

Old §2 notes follow.

#499 deleted the `ci-gate` failure that forced `specsync change finalize`. That failure *was* the
tip dance, so removing it was right — but nothing replaced it. CHG-0079 merged unfinalized; CHG-0080
will too. Active changes accumulate on main and stale.

**Decision needed:** post-merge finalize job on main (recommended — keeps merges clean, makes
finalization inevitable) vs documented pre-merge step.

## §3 — Approval digest invalidation

Editing artifacts after approval silently invalidates the approval digest; the author discovers it
at the next gate. Per working style: **warn at the edit, not at the gate.** Needs a cheap
`change status` signal, or a hook.

## §4 — #495 TODO/placeholder artifacts

`# TODO` headings pass scope approval while prose containing "TODO" false-positives. Product-side,
unaffected by #499.

---

## Closed by deletion (verify before planning work on them)

#496, #497, #498 all target files deleted in #499
(`reuse-check-from-ancestors.py`, `post-merge-archive.yml`, `lifecycle-policy-guard.yml`).
#487 "buttery ship" premise removed — `ship-status` dropped, tip dance deleted.

---

## Sequence

1. Land #500 (three fixes + §1 guard) — unblocks every subsequent PR
2. §2 finalization trigger — required before a clean tag or changes pile up on main
3. §3 approval-digest warning
4. #495
5. Sandbox candidate CI: unpin from v5.2.0, build from candidate SHA, `SKIP` must stop counting as
   `PASS` — until then nothing exercises 6.0 behaviour
6. Two consecutive clean PRs with zero spec-sync-reason failures → tag

## Elevate to human

- §1 open question, once validation lands
- §2 decision
- Any change to digest/acceptance semantics — twice this session a fix there looked right and
  contradicted a deliberate invariant, caught only by the full suite
- Anything needing an approval digest, scoped review, or admin merge
