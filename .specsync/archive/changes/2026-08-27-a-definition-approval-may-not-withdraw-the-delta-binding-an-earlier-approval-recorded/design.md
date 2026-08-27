---
change: a-definition-approval-may-not-withdraw-the-delta-binding-an-earlier-approval-recorded
artifact: design
---

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
