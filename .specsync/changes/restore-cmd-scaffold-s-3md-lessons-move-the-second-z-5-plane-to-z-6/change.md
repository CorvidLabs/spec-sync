---
id: restore-cmd-scaffold-s-3md-lessons-move-the-second-z-5-plane-to-z-6
state: draft
type: bug_fix
base_commit: ad8544e9da79725f499b38ae4d4b3a052be5f807
---

# Restore cmd_scaffold's 3md lessons: move the second z=5 plane to z=6

## Intent

Restore cmd_scaffold's 3md lessons: move the second z=5 plane to z=6

## Affected Canonical Specs

- `cmd_scaffold`

## Acceptance Criteria

- specs/cmd_scaffold/cmd_scaffold.3md has unique plane positions (z=0 to z=6) and parses, so a change briefing for cmd_scaffold reads its lessons from cmd_scaffold.3md instead of falling back to context.md; check --strict still passes

## No-spec Rationale

A duplicate z makes threemd refuse the file, so cmd_scaffold's lessons silently fall back to context.md. Renumbering one plane restores the existing lesson source; no contract or requirement text changes.
