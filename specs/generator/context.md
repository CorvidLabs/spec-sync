---
spec: generator.spec.md
---

## Key Decisions

- Generation is deterministic and local.
- Custom templates override built-ins without overwriting existing files.
- Companion creation and module discovery remain independent of coding-agent enrichment.
- CLI generate holds one project-root capability through template reads, directory creation, and
  no-overwrite publication; public-path checks detect replacement but never authorize writes.

## Files to Read First

- `src/generator.rs`
- `src/exports/mod.rs`
- `src/types.rs`

## Current Status

Stable local scaffold generator with no provider, credential, network, or shell path. CLI
publication is capability-relative and cannot follow a later public-root redirect.
