---
spec: generator.spec.md
---

## Key Decisions

- Generation is deterministic and local.
- Custom templates override built-ins without overwriting existing files.
- Companion creation and module discovery remain independent of coding-agent enrichment.

## Files to Read First

- `src/generator.rs`
- `src/exports/mod.rs`
- `src/types.rs`

## Current Status

Stable local scaffold generator with no provider, credential, network, or shell path.
