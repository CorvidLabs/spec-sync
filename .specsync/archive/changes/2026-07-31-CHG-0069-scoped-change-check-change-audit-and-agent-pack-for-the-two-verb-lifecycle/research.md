---
change: CHG-0069-scoped-change-check-change-audit-and-agent-pack-for-the-two-verb-lifecycle
artifact: research
---

# Research

## Observed problem

On this monorepo, `specsync change check CHG-0068` spent ~20–25 minutes printing
`evidence: authenticated-history` for ~72 archived changes before `cargo test` started.
CLI wiring always ran `check_change` **and** full `check_project` (active + archive terminal
evidence).

## Product interview

- Default check = this change only
- Archives = history; living truth in specs + active workspaces
- Full integrity = separate `change audit` (actives + living specs only)
- Hard cut (no dual deprecation window)
- Agent pack: skill + `/specsync:check` + `/specsync:audit`
- TS/sh SDK wrappers deferred

## Related surfaces

Existing `agents install` already generated create-spec/create-change; check/audit follow the
same template + digest manifest pattern.
