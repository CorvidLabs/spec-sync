---
id: CHG-0043-make-accepted-change-validity-successor-aware-with-exact-per-input-evidence-rec
state: implementing
type: bug_fix
base_commit: fc6e70bccd5af61043183e247f37b1f9a9b92247
---

# Make accepted-change validity successor-aware with exact per-input evidence, recursive cycle-safe validation, fail-closed legacy compatibility, and safe archived successors

## Intent

Make accepted-change validity successor-aware with exact per-input evidence, recursive cycle-safe validation, fail-closed legacy compatibility, and safe archived successors

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cli_args`

## Acceptance Criteria

- New closing evidence signs bounded canonical per-input topology digests and a versioned acceptance manifest while legacy raw-content evidence remains byte-compatible.
- A supported pre-approval supersede transition records durable definition-bound predecessor path, module, and predecessor-entry obligations.
- Semantic succession evidence is bounded, strictly ordered, one-to-one with approved obligations, and binds trusted old and new tree entries.
- Every changed predecessor input requires terminal same-successor coverage for each canonical owner; exact-only delivery and test inputs never accept inferred succession.
- Successor validity is recursive and cycle-safe and requires accepted or authenticated archived state with current definition, verification, closing, and semantic evidence.
- Stale legacy evidence is usable only through unique trusted historical reconstruction; enumerated standalone archives use a definition-bound immutable baseline for historical integrity only.
- Check, status, reopen, and archive share location-aware fail-closed validation with authenticated accepted snapshots and retry-safe archive preflight.
- Focused and full regression suites cover topology, tuple security, legacy ambiguity, archive history, recursive successors, and immediate uncommitted archive status.

## No-spec Rationale

Not applicable
