---
change: a-definition-approval-may-not-withdraw-the-delta-binding-an-earlier-approval-recorded
artifact: docs
---

# Docs

No user-facing documentation page changes. The CLI surface is unchanged: `change approve` and
`change approve --portable-5-0-1` take the same flags and still record one atomic marked pair,
which is what `specs/cmd_change/requirements.md` states about the flag.

What changed is recorded evidence and one refusal, and both are documented where they are
enforced:

- `specs/change/change.spec.md` — contract clause 3 and invariant 40 state the monotonicity rule;
  the Error Cases table names the withdrawn-claim refusal and its remedy.
- `specs/change/context.md` — the module's standing lesson about `approved_delta_digests` now
  records that absence is a property of a ledger, not of an event.
- `src/change.rs` — the invariant is stated at the `approved_delta_digests` declaration, beside
  the existing "absent evidence is not a violation" note, so the next reader of the field sees
  both halves at once.

Operators who hit the new refusal are told exactly what to run (`specsync change approve <id>`);
no separate migration or guide is needed, and none is added. The 5.0 ledger migration
(`specsync migrate 5.0`) is untouched: no existing ledger carries the refused shape, so there is
nothing to backfill.
