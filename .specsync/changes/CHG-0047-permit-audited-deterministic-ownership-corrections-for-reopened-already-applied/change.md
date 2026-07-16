---
id: CHG-0047-permit-audited-deterministic-ownership-corrections-for-reopened-already-applied
state: accepted
type: bug_fix
base_commit: 2223c0b2ba260c43c396c195885ffe727f2d69e8
---

# Permit audited deterministic ownership corrections for reopened already-applied changes

## Intent

Permit audited deterministic ownership corrections for reopened already-applied changes

## Affected Canonical Specs

- `change`
- `cli_args`
- `cmd_change`

## Acceptance Criteria

- A supported change correct-owner transition adds one exact path/module ownership correction only to an audited reopened canonical-applied change; the path is already in scope and the module canonically owns it at the trusted current tree; the transition records a non-empty human actor and reason without changing affected specs, semantic deltas, prior approvals, or reopen evidence; the corrected definition requires explicit reapproval, fresh verification, and closing approval; acceptance includes the corrected owner in its signed manifest without replaying canonical deltas; removals, duplicates, non-owners, out-of-scope paths, un-reopened changes, malformed paths/modules, stale approvals, and tampering fail transactionally; legacy records with no corrections remain byte-compatible
- An accepted legacy archive baseline authority signs the exact protected baseline ledger path in its manifest while unrelated dated archive paths remain excluded

## No-spec Rationale

Not applicable
