---
change: CHG-0069-scoped-change-check-change-audit-and-agent-pack-for-the-two-verb-lifecycle
artifact: context
---

# Context

## Decision

`specsync change check` is scoped verification for one change. Project health for **active**
workspaces and living specs is `specsync change audit`. Archives are history and are not
re-validated on the daily path.

## Why

Full `check_project` rewalk of archived terminal evidence took ~20–25 minutes on this repo
before cargo tests even started. That dual-wiring was 5.x accumulation, not a 6.0 workflow-v2
requirement. Agents and humans both paid the tax on every `change check`.

## Key files

- `src/commands/change.rs` — CLI check/audit handlers
- `src/change.rs` — `audit_project` / scoped `check_project` options
- `src/cli.rs` — `ChangeAction::Audit`
- `src/agents.rs` — skill + `/specsync:check` + `/specsync:audit`
- `src/hooks.rs` — snippet two-verb model
