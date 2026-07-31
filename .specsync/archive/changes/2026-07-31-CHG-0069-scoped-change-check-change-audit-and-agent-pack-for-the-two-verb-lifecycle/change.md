---
id: CHG-0069-scoped-change-check-change-audit-and-agent-pack-for-the-two-verb-lifecycle
state: archived
type: feature
base_commit: 4c32e08e3bfaffb1ab121c85069c3638e842e544
---

# Scoped change check, change audit, and agent pack for the two-verb lifecycle

## Intent

Scoped change check, change audit, and agent pack for the two-verb lifecycle

## Affected Canonical Specs

- `cli`
- `cmd_change`
- `change`
- `agents`
- `hooks`
- `commands`
- `cmd_agents`
- `cli_args`

## Acceptance Criteria

- change check verifies only the target change without revalidating archived terminal evidence; change audit validates active workspaces and living specs only; agents install ships /specsync:check and /specsync:audit; skill and hooks teach the two-verb model; text UX shows phases and a one-line success footer with next action

## No-spec Rationale

Not applicable
