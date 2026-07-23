---
spec: config.spec.md
---

## Key Decisions

- Current and legacy JSON/TOML layouts remain readable for migration.
- Retired AI key names are recognized only to emit value-safe migration guidance, then ignored.
- Configuration never interprets provider credentials or commands.
- Source discovery recognizes supported language files plus default measurable HTML, HTM, and CSS content at the root or within top-level directories while preserving ignored-directory and empty-project behavior.
- Checked source-directory and manifest discovery surface malformed or unreadable Gradle settings;
  existing infallible entry points remain compatibility wrappers, with scan fallback for source dirs.
- A present legacy JSON `github.repo` accepts only a string (or explicit `null` as absent).
  Numbers, booleans, objects, and lists retain an invalid sentinel so repository resolution fails
  closed instead of falling back to Git auto-detection.
- Retained filesystem callers use the crate-private checked snapshot parser: JSON/TOML is parsed
  from exact supplied bytes, real TOML syntax and known field types are validated first, and no
  pathname is reopened.
- Capability callers can supply source-directory discovery from their retained project handle, so
  an omitted source list does not trigger ambient root traversal or observe a path replacement.

## Files to Read First

- `src/config.rs`
- `src/types.rs`

## Current Status

Stable 5.0 secret-free configuration schema with checked discovery available to validation gates.
CHG-0063 adds fail-closed legacy GitHub repository shape validation, exact-byte checked parsing,
and capability-supplied source discovery for issue inspection. The exact checked JSON path rejects
a non-object `github` value and a non-string/non-null `github.repo` before the compatibility parser
can substitute an invalid sentinel.
