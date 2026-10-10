---
id: split-change-command-dispatch-rendering-shipping-and-lifecycle-helpers-into-private-modules
state: implementing
type: refactor
base_commit: 815690d1b01776915d93d11ff9f814efeced1a3d
---

# Split change command dispatch rendering shipping and lifecycle helpers into private modules

## Intent

Split change command dispatch rendering shipping and lifecycle helpers into private modules

## Affected Canonical Specs

- `cmd_change`

## Acceptance Criteria

- Keep the change command easier to navigate by splitting the entire dispatcher implementation and all existing tests into cohesive private modules below 1000 lines wherever coherent boundaries permit; preserve the cmd_change API and all current function bodies, literals, cfg guards, snapshot-bound mutation output, read-scope lifetimes, degraded-roster exits, domain readiness predicates, scoped literal NUL-aware lifecycle staging and ledger flooring, gate-before-push PR orchestration and no automatic merge; retain every regression and canonical source coverage without weakening lifecycle or trust gates

## No-spec Rationale

Not applicable
