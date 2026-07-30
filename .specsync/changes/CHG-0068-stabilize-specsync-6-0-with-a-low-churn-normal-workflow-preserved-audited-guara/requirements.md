---
change: CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara
artifact: requirements
---

# Requirements

## R1 — One discoverable workflow

Every change SHALL use one file layout, one human scope approval, bidirectional implementation
checks, ordinary PR review, same-PR finalization, and GitHub merge protections.

## R2 — No hidden lifecycle fork

No user-facing lifecycle mode, second SpecSync approval, closing gate, alternate archive layout, or
SpecSync merge command SHALL exist. Explicit `--strict`, project policy, and release/security
classification SHALL add powerful validators to the same evidence only.

## R3 — Targeted evidence

Implementation verification SHALL run deterministic affected-component commands and record concise
digest-bound outcomes. Missing mappings SHALL use a bounded fallback rather than report success.

## R4 — Same-PR finalization

`change finalize` SHALL apply canonical deltas and produce the only archive commit on the original
PR. It SHALL make the PR ready for GitHub merge without performing that merge.

## R5 — Lightweight archive integrity

Archive-only CI SHALL verify the parent implementation checks, exact approved lifecycle diff,
unchanged delivery tree, archive integrity, and bidirectional ownership, then report to required CI.
It SHALL NOT rerun the full product matrix or scoped agent review.

## R6 — Bounded validation

Lifecycle checking SHALL reuse one deterministic invocation snapshot, bound Git/evidence queries,
and validate canonical owner batches in one pass without weakening historical integrity.

## R7 — Independent scoped review

Agent-authored work SHALL receive one independent review of its package, implementation diff,
canonical spec delta, and targeted evidence before finalization. Status SHALL explain when and how
to request it, and finalization metadata SHALL not trigger it again.

## R8 — Plain status guidance

Every `change status` result SHALL print exactly one next action consistent with the documented
`new → approve → implement → check → review → finalize → GitHub merge` path.

## R9 — Focused reliability fixes

The 6.0 integration SHALL include reviewed schema replay, ignore suppression, TypeScript/Erlang
export, and managed hook/agent safety fixes with focused characterizations and regressions.

## R10 — Context discipline

Canonical specs SHALL change only for durable behavior or decisions. Transient logs SHALL remain
concise, and retrieval SHALL select current affected contracts plus relevant decisions.
