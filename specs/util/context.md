---
spec: util.spec.md
---

## Key Decisions

- **Centralized helpers**: Edit-distance and safe-regex helpers live in `util` to avoid duplicate implementations in parser and validator code.
- **Fail closed regex compilation**: User-provided regex patterns return `None` when invalid or too large, keeping callers on the safe path without panics.

## Files to Read First

- `src/util.rs` — Shared helper functions and unit tests.
- `src/validator.rs` — Main caller for regex-safe config patterns and near-miss suggestions.

## Current Status

Stable. The helpers are small, covered by unit tests, and intentionally have no project-internal dependencies.

## Notes

- Keep this module dependency-light. It is imported by validation code that runs on every check.
