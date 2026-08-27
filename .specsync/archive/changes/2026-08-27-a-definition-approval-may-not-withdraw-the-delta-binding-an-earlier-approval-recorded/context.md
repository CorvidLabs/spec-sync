---
change: a-definition-approval-may-not-withdraw-the-delta-binding-an-earlier-approval-recorded
artifact: context
---

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
