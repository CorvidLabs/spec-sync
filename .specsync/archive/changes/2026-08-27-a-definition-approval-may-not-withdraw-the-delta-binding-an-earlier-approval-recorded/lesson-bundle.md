# Lesson bundle — a-definition-approval-may-not-withdraw-the-delta-binding-an-earlier-approval-recorded

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: A definition approval may not withdraw the delta binding an earlier approval recorded
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change.rs, src/change_tests.rs, specs/change/change.spec.md, specs/change/requirements.md, specs/change/context.md, specs/change/tasks.md, specs/change/testing.md
- **Acceptance**: change approve --portable-5-0-1 records the delta wording it approves, so the effective definition approval of a change that already recorded delta digests never drops to no claim; a ledger whose latest definition approval records no delta digests while an earlier definition approval did is refused at materialization and acceptance with a message naming the re-approve remedy; a ledger in which no definition approval ever recorded a delta digest still materializes unchanged

## Evidence

- Verification commit: `9ee16607822f3a9e3219881a3a0eeefff6ea46f1`
- Base commit: `62b297a4eb1822ec444460a172d6264317ebbf2e`
- Verified by: `cargo test change::`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

# Context

## What led here

#711 bound semantic delta bodies to the definition approval that signed them.
`approve` records `ApprovalRecord.approved_delta_digests` — one digest per module over the
delta file's exact bytes — and `ensure_approved_delta_bodies_unchanged` re-checks it before
materialization and before acceptance. A body swapped after approval is refused.

An independent review of rc.7 asked whether the *effective* approval could be made to carry no
digest while an earlier approval on the same change carried one. It can, and #719 reports it.

`change approve --portable-5-0-1` runs `approve_definition_portable_v501` →
`approve_definition_with_projection(.., portable_v501: true)` →
`append_portable_definition_approval_v501`, which appended **two** `ApprovalRecord`s, both
`gate: "definition"`, both with `approved_delta_digests: None`.
`effective_definition_approval` selects the LAST definition-gate event with `rposition`, so:

1. `change approve` records `Some({module: digest})`.
2. `change approve --portable-5-0-1` appends two records carrying `None`.
3. The effective definition approval now carries `None`, so
   `ensure_approved_delta_bodies_unchanged` hits its `let Some(..) else { return Ok(()) }`
   and returns immediately.
4. The binding #711 added is disarmed for the rest of that change's life.

## The constraint this change is bounded by

The absent-digest path is **correct and stays**. Every approval written before the field existed
carries `None`, which is every archived change in this repository. `None` means "this approval
made no claim about delta wording", never "the bodies were tampered with". Making absence fail
closed would fail all of recorded history on evidence nobody could have written — the shape this
project has already fixed in #672 (an unparseable schema reported every table as missing) and
#684 (a missing config gated a release on advice the reader could not take), and the shape #711
was deliberately built to avoid.

What is wrong is narrower and exactly statable: a **later** approval can reintroduce that state
on a change that already had a digest. Absence must keep meaning "this approval predates the
binding"; it must never come to mean "this approval declines to make a claim".

## What was measured before choosing

Two facts were established against the unfixed binary rather than assumed, and they set the
scope of what this change can honestly claim.

- **The downgrade is real.** On a workflow-v1 change, `approve` then
  `append_portable_definition_approval_v501` leaves the effective approval's
  `approved_delta_digests` at `None` after it had been `Some({"auth": ..})`.
- **On workflow v1 the downgrade does not currently reach the canonical spec.** The v1 definition
  digest hashes every delta payload through `definition_artifact_snapshot`, so a swapped body is
  independently caught one line earlier, by `ensure_definition_approval_valid` inside
  `materialize_change_deltas`. Unfixed, that sequence fails with
  `portable definition approval pair is malformed or stale`.

So on v1 the portable downgrade costs *recorded evidence*, not a materialization. That is not a
reason to leave it: the message a reader gets today ("the pair is stale") points at re-running
`--portable-5-0-1`, which re-approves the swapped wording and again records no claim about it.
And the shape itself — a later definition approval that records nothing where an earlier one
recorded a digest — is, under workflow v2, the *whole* of what stands between a swapped body and
the canonical spec, because the v2 scope digest deliberately hashes intent and boundary only.
That is the thing that had to be refused, and it is what the discriminator tests pin.

## Already ruled out

- **Refusing `--portable-5-0-1` when a digest-bearing approval exists.** Cheaper, and it does
  enforce monotonicity — but it removes the only way an adopter can hand a 5.0.1 verifier a
  change the current binary has already approved, with no workaround short of editing the ledger
  by hand. See design.md.
- **Falling back to the most recent digest-bearing approval when the effective one is silent.**
  That reads one event's evidence under a later event's authority and leaves the untruthful
  ledger in place. Refusing and naming `change approve` restores a ledger that says what
  happened.
- **Making absence fail closed.** Ruled out on arrival; see the constraint above.

## From the change's design.md

# Design

## The rule

**Monotonicity.** Once an approval ledger has recorded a delta digest for a change, no later
definition approval on that change may record none.

Absence of `approved_delta_digests` is a statement about a *ledger*, not about one event: a
ledger that has recorded a digest for this change is demonstrably new enough to record one
again, so a later definition approval carrying none is not silence from before the binding
existed — it is a claim being withdrawn.

The rule is enforced on both sides, because they answer different questions.

## Write side: carry the digest forward (the decision)

`append_portable_definition_approval_v501` now records `approved_delta_digests` on both members
of the pair, computed from the delta bodies on disk at approve time.

The alternative in #719's own wording was to **refuse** the portable approve when a
digest-bearing definition approval already exists. Carrying forward was chosen for four reasons,
in order of weight.

1. **It is what a definition approval already means here.** `append_approval` records
   `approved_delta_digests` for *every* `definition` gate, with the comment that only a definition
   gate approves wording. The normalizing definition approval inside `accept_change` already
   carries the binding forward rather than dropping it, and says so in a comment. The portable
   path was written before #711 and was simply never updated; `None` there was an omission, not a
   position. Refusing would instead make `--portable-5-0-1` permanently the one definition
   approval in the system that declines to say what it approved.
2. **Refusing removes a capability with no workaround.** `--portable-5-0-1` exists so an adopter
   upgrading from 5.x can hand a 5.0.1 verifier an approval it can check. The sequence
   `change approve` → `change approve --portable-5-0-1` is the normal way to reach it on a change
   the current binary already approved. Refusing that leaves hand-editing `approvals.json` as the
   only route — which is the state this whole binding exists to make unnecessary.
3. **The claim is honest.** The bodies are read inside the approve call, after
   `validate_delta_files`, so the digest records the wording *this* actor is approving now. It is
   not a claim inherited from an older event and re-signed on someone else's behalf.
4. **Monotonicity then falls out rather than being bolted on.** Every writer of a definition gate
   records what it signed, so within one ledger the field only ever goes absent → present.

**The portable records' other digests are unaffected, and this is checked rather than assumed.**
`approved_delta_digests` is an input to none of `definition_digest`, the 5.0.1 projection bytes
(`definition_projection_bytes_v501` projects the change record, not the ledger), or
`definition_approval_pair_id`. `resolve_definition_approval_event` compares gate, actor,
timestamp, digest and `definition_pair` metadata, and none of those change.
`a_portable_definition_approval_carries_the_delta_binding_it_inherits` asserts the pair's
`current`/`legacy` digests still equal `portable_definition_digest_pair_v501` and that
`ensure_definition_approval_valid` still resolves.

**Portability is preserved.** `ApprovalLedger` is deliberately tolerant of unknown fields — the
module says so at length, and names this exact scenario as the reason: an older binary must be
able to read a file a newer binary wrote. A 5.0.1 reader parses the pair it came for and ignores
a field it does not know. `skip_serializing_if` also means the field appears only where it is
recorded, so nothing about existing ledgers changes shape.

## Read side: refuse a withdrawn claim

`ensure_approved_delta_bodies_unchanged` keeps returning `Ok(())` on absence — that is the
compatibility path and it is untouched. It now qualifies that reading with one scan: if the
effective definition approval records no delta wording **and** some definition approval in the
same ledger does, the ledger has been walked back, and it is refused.

Why the read side is needed at all when the write side can no longer produce the shape: the
invariant is a property of ledgers, and ledgers arrive from other binaries, other branches, and
hand edits. Fixing only the writer leaves the reader trusting a state that means the opposite of
what it reads as.

The refusal names the remedy, in the style the neighbouring refusal already uses:

> the definition approval for `<id>` records no semantic delta wording although an earlier
> definition approval on this change recorded it; an approval cannot withdraw a claim an earlier
> one made, so re-run `specsync change approve <id>` to record the delta bodies this approval
> covers (or restore the approval ledger)

The scan is restricted to `gate == "definition"`. Closing and finalization gates record `None` by
design — claiming they reviewed delta bodies would be a lie recorded in the ledger — so including
them would make every accepted change violate the rule.

## Blast radius on existing evidence: measured, zero

All 197 approval ledgers under `.specsync/` (archive and active) were scanned for the shape the
new refusal rejects — a definition approval carrying a digest followed by a later definition
approval carrying none. **Zero** ledgers match. Archived changes carry no digest on any
definition approval at all, so they take the untouched absence path exactly as before.

## From the change's testing.md

# Testing

Four tests in `src/change_tests.rs`, beside the #711 tests they extend. Each was run with the fix
**disabled in place** — the two carry-forward assignments reverted to `None`, the monotonicity
scan short-circuited — to establish which of them are discriminators and which is not.

| Test | Unfixed | Fixed |
|------|---------|-------|
| `a_portable_definition_approval_carries_the_delta_binding_it_inherits` | FAIL: effective approval is `None` where `Some({"auth": 66d9882e..})` had just been recorded | pass |
| `a_later_definition_approval_may_not_withdraw_a_recorded_delta_binding` | FAIL: no error at all — `materialize_change_deltas` returned `Ok`, `canonical_applied: true`, and `specs/auth/auth.spec.md` contained `BACKDOOR` | pass |
| `a_portable_definition_approval_records_delta_wording_with_no_prior_approval` | FAIL: the portable approve records no wording | pass |
| `a_ledger_that_never_recorded_delta_wording_still_materializes_a_swapped_body` | **pass** | pass |

Every other test in the file passed in both states — including
`an_approval_recorded_before_delta_digests_existed_is_unknown_not_violated`,
`a_semantic_delta_swapped_after_approval_never_reaches_the_canonical_spec`, and the four
`portable_definition_*` tests that pin the pair's shape.

## Honest labels

- The **control** is `a_ledger_that_never_recorded_delta_wording_still_materializes_a_swapped_body`,
  and it passes on the unfixed binary too. That is the point of writing it. It builds the ledger
  monotonicity is easiest to get wrong on — a pre-#711 change approved more than once, so several
  definition approvals, not one of them carrying a digest — swaps the body, and requires the
  materialization to succeed. If the refusal is ever written as "the latest approval records
  nothing" instead of "an earlier approval recorded more", this test fails, and it fails as an
  outage across every archived change rather than as a caught bug.
- `a_later_definition_approval_may_not_withdraw_a_recorded_delta_binding` writes the downgraded
  ledger **directly** rather than through `change approve --portable-5-0-1`, and the test says so
  in its own doc comment. The portable projection is workflow-v1-only, and a v1 definition digest
  hashes every delta body, so on a v1 change a swapped delta is independently caught one line
  earlier by `ensure_definition_approval_valid` — unfixed, that sequence produces
  `portable definition approval pair is malformed or stale`, which is a refusal, just not this
  one. What generalizes is the shape, and under workflow v2 the shape is the whole of what stands
  between a swapped body and the canonical spec. The test asserts on the canonical spec's
  contents, not only on a message.
- `a_portable_definition_approval_carries_the_delta_binding_it_inherits` also pins the half that
  must NOT change: the pair's current and legacy digests still equal
  `portable_definition_digest_pair_v501`, the legacy note is unchanged, and
  `ensure_definition_approval_valid` still resolves the pair.

## Suites run

- `cargo clippy -- -D warnings` (bare — the form CI runs; `change check` does not run clippy)
- `cargo test` — 2395 unit tests plus 407 integration tests, all passing
- `specsync change check` and `specsync change audit --strict`

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-090 | Four tests in `src/change_tests.rs`, each run with the fix disabled in place to separate discriminator from control. `a_portable_definition_approval_carries_the_delta_binding_it_inherits` — unfixed: effective approval is `None` where `Some({"auth": 66d9882e..})` had just been recorded; it also pins that the pair's current/legacy digests still equal `portable_definition_digest_pair_v501`, that the legacy note is unchanged, and that `ensure_definition_approval_valid` still resolves. `a_later_definition_approval_may_not_withdraw_a_recorded_delta_binding` — unfixed: no error at all; `materialize_change_deltas` returned `Ok` with `canonical_applied: true` and `specs/auth/auth.spec.md` contained `BACKDOOR`. `a_portable_definition_approval_records_delta_wording_with_no_prior_approval` — unfixed: the portable approve records no wording; fixed: it records `{"auth": ..}` and both `ensure_definition_approval_valid` and `ensure_approved_delta_bodies_unchanged` pass. CONTROL `a_ledger_that_never_recorded_delta_wording_still_materializes_a_swapped_body` passes in BOTH states, which is its purpose: a pre-binding ledger holding several silent definition approvals, body swapped, materialization required to succeed. Blast radius measured rather than argued: all 197 `approvals.json` files under `.specsync/` were scanned for the refused shape and none matches. `cargo clippy -- -D warnings` bare, 2395 unit tests and 407 integration tests pass |

## Where these lessons go

- `specs/change/context.md`
