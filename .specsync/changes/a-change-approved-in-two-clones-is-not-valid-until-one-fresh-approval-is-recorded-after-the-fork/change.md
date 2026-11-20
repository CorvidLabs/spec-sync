---
id: a-change-approved-in-two-clones-is-not-valid-until-one-fresh-approval-is-recorded-after-the-fork
state: implementing
type: bug_fix
base_commit: 9b75e8baac2c1b254fa77ee8f8a053ed74a12f05
---

# A change approved in two clones is not valid until one fresh approval is recorded after the fork

## Intent

A change approved in two clones is not valid until one fresh approval is recorded after the fork

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Two clones approve the same change with different actors and each commits. After a merge that keeps one clone's ledger, or both, `change check` and `change audit --strict` refuse, and `change status` reports the approval as not valid, naming both approvers, both commits and the fresh approval.
- One `change approve` on the merged branch settles it, and the same commands pass.
- Approvals that only append are not a fork, and neither is a change sent back and approved again, or a relay removed or restored by `change repair-relays`.
- An archived change whose history holds a fork is not judged again.
- Through the binary, the reproduction above is refused, and one fresh approval lets `change check --commit` pass.

## No-spec Rationale

Not applicable
