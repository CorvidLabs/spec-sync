---
id: repair-relays-also-repairs-relays-nested-in-reopenings-and-corrections
state: implementing
type: feature
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Repair-relays also repairs relays nested in reopenings and corrections

## Intent

repair-relays also repairs relays nested in reopenings and corrections

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- repair-relays puts back, or removes, a damaged relay in an approvals.json reopening's superseded_approval and in each approval a corrections.json correction embeds, keeping every reopening and correction; it prints approvals.json reopening <n> superseded approval and corrections.json correction <n> definition|closing approval, and --json names the nested record in within (red then green)

## No-spec Rationale

Not applicable
