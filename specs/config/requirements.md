---
spec: config.spec.md
---

## User Stories

- As a developer, I want zero-config source discovery and JSON/TOML compatibility.
- As a team lead, I want deterministic validation, lifecycle, and module settings.
- As a security-conscious maintainer, I want retired inference keys ignored without exposing their values.

## Constraints

- Loading is local and performs no network calls or command execution.
- Existing legacy layout formats remain readable for migration.
- Present but unreadable or malformed configuration fails loud before fallback.

### REQ-config-001

Configuration loading SHALL never interpret repository or local configuration as inference credentials or executable AI commands.

Acceptance Criteria
- AI configuration fields are removed from JSON/TOML readers and writers.
- Legacy AI keys produce migration guidance without activating behavior.
- The obsolete AI-only local override merge is removed.

### REQ-config-002

Configuration source-directory autodetection SHALL recognize default measurable static files in addition to language exports.

Acceptance Criteria

- Static-only root projects resolve to `.`.
- Static-only nested projects resolve to the containing top-level directory.
- Empty projects retain the `src` fallback.
