---
spec: config.spec.md
---

## Key Decisions

- Current and legacy JSON/TOML layouts remain readable for migration.
- Retired AI key names are recognized only to emit value-safe migration guidance, then ignored.
- Configuration never interprets provider credentials or commands.
- Source discovery recognizes supported language files plus default measurable HTML, HTM, and CSS content at the root or within top-level directories while preserving ignored-directory and empty-project behavior.

## Files to Read First

- `src/config.rs`
- `src/types.rs`

## Current Status

Stable 5.0 secret-free configuration schema.
