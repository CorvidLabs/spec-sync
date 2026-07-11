---
spec: config.spec.md
---

## Key Decisions

- Current and legacy JSON/TOML layouts remain readable for migration.
- Retired AI key names are recognized only to emit value-safe migration guidance, then ignored.
- Configuration never interprets provider credentials or commands.
- Source discovery and deterministic validation/lifecycle settings remain unchanged.

## Files to Read First

- `src/config.rs`
- `src/types.rs`

## Current Status

Stable 5.0 secret-free configuration schema.
