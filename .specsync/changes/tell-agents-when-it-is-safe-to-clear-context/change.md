---
id: tell-agents-when-it-is-safe-to-clear-context
state: implementing
type: feature
base_commit: ddbc9343fa30be3a2def39f2e559ca9cf6984d2c
---

# Tell agents when it is safe to clear context

## Intent

Tell agents when it is safe to clear context

## Affected Canonical Specs

- `change`
- `cmd_change`
- `agents`

## Acceptance Criteria

- Every change status, show, check, approve, review, and finalize text result prints exactly one Handoff line that says safe, conditional, or not yet, gives one reason in plain language without digests, and when it is not safe names the concrete steps to take before clearing context.
- The JSON summary carries the same decision under summary.handoff with readiness, reason, resume, and before_clearing.
- Uncommitted edits under the change affected_paths make the handoff conditional; uncommitted lifecycle evidence under .specsync/ alone does not, because review then finalize is designed to run with that evidence uncommitted.
- A Draft change is never reported as safe; approval is the first clean boundary, and an approved change with a clean tree and current evidence is safe.
- A stale approval digest, a frozen sequence ledger, an invalid correction ledger, and stale legacy terminal evidence are all reported as not yet with the repair named.
- The handoff decision is a pure function of gathered lifecycle signals with a unit test per branch, and the installed agent skill tells agents to clear context only when the Handoff line says safe.

## No-spec Rationale

Not applicable
