---
spec: config.spec.md
---

## Key Decisions

- Current and legacy JSON/TOML layouts remain readable for migration.
- Retired AI key names are recognized only to emit value-safe migration guidance, then ignored.
- Configuration never interprets provider credentials or commands.
- Source discovery recognizes supported language files plus default measurable HTML, HTM, and CSS content at the root or within top-level directories while preserving ignored-directory and empty-project behavior.
- Callers that describe detection use `detect_source_dirs_with_confidence` so the compatibility `src` fallback is never reported as discovered evidence.
- Mutating initialization commands call `validate_config_file` before repair/generation. Unknown extension keys remain allowed, while malformed syntax and wrong known path-field shapes fail before writes.
- Canonical TOML escaping covers all control characters and quoted key components.

## Files to Read First

- `src/config.rs`
- `src/types.rs`

## Current Status

Stable 5.0 secret-free configuration schema with checked mutation preflight and truthful source-detection metadata.
