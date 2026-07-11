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
