---
change: a-definition-approval-may-not-withdraw-the-delta-binding-an-earlier-approval-recorded
artifact: research
---

# Research

## Where the state comes from

| Site | Gate | `approved_delta_digests` written |
|------|------|----------------------------------|
| `append_approval` | `definition` | `Some(delta_body_digests(..))` |
| `append_approval` | closing / finalization | `None`, deliberately — those gates bind delivery evidence and never reviewed wording |
| `accept_change`, normalizing approval | `definition` | `Some(..)`, with a comment saying it carries the binding forward rather than dropping it |
| `accept_change`, closing/finalization entries | closing / finalization | `None`, same reason as above |
| `append_portable_definition_approval_v501` | `definition` ×2 | `None` — **the defect** |

`effective_definition_approval` picks the last `definition`-gate event
(`.rposition(|approval| approval.gate == "definition")`), so the portable pair's silence became
the change's answer. Every other definition writer already recorded a claim, which is why the
fix is "make this one behave like the others" rather than a new rule.

## Scope of the portable path

`portable_definition_digest_pair_v501_with_task_mode` refuses unless
`record.workflow_version == 1`, `canonical_applied` is false, `correction_count == 0`,
`supersedes` is empty, `acceptance_owner_corrections` is empty, the correction ledger is empty,
and a versioned legacy archive baseline binding is present. So only workflow-v1 records reach it
— which bounds the defect, and is also exactly the population `--portable-5-0-1` exists for: an
adopter upgrading from 5.x.

## Whether the downgrade reaches a canonical spec today — measured

Run against the unfixed binary, workflow-v1 fixture with an `auth` delta:

| Step | Result |
|------|--------|
| `approve_definition` | effective approval carries `Some({"auth": ..})` |
| `append_portable_definition_approval_v501` | effective approval carries `None` |
| `ensure_definition_approval_valid` before swap | `Ok(())` |
| swap `deltas/auth.md`, then `ensure_definition_approval_valid` | `Err("portable definition approval pair is malformed or stale")` |
| swap, then `materialize_change_deltas` | same `Err`, spec untouched |

So on workflow v1 the swap is caught anyway, by the definition digest itself: v1 hashes every
delta payload through `definition_artifact_snapshot`, and `materialize_change_deltas` runs
`ensure_definition_approval_valid` one line before `ensure_approved_delta_bodies_unchanged`.

Two things follow, and both are in the fix.

1. The consequence on v1 today is to the *ledger's evidence*, and to the diagnostic. The message
   a reader gets blames the approval pair, so the indicated repair is to re-run
   `--portable-5-0-1` — which re-approves the swapped wording and again records no claim about
   it. The tool guides the operator into laundering the swap.
2. The consequence in general is a materialization. Under workflow v2 the effective approval's
   digest is the *scope* digest — intent and boundary only — so `approved_delta_digests` is the
   only thing binding wording. The same downgraded shape on a v2 ledger materializes a body
   nobody approved. Confirmed against the unfixed binary: a v2 change with a duplicate definition
   approval whose delta claim is dropped materializes `BACKDOOR` wording into
   `specs/auth/auth.spec.md` and returns `Ok`.

## Compatibility surface

- `ApprovalLedger` and `ApprovalRecord` are deliberately **tolerant** of unknown fields; the
  module documents this as the mechanism by which evidence shapes can grow inside 6.x without a
  major version. A 5.0.1 reader therefore parses a record carrying `approved_delta_digests` and
  ignores it.
- The three digest-preimage structs that adding a field *would* disturb are `ApprovedScopeV1`,
  `CorrectionRecord` and `ScopedReviewRecord`. `ApprovalRecord` is none of them, and no digest in
  the portable pair is computed over an `ApprovalRecord`'s serialized bytes.
- `.specsync/**` ledger scan: 197 `approvals.json` files, 0 carrying the refused shape.
