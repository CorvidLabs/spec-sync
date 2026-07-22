---
spec: cmd_generate.spec.md
---

## Key Decisions

- `generate` is deterministic and local; no provider/model/config/env branch exists.
- All and batch modes delegate to `generator`, then recompute validation and coverage.
- JSON returns stable generated/requested/skipped fields.
- Coverage selection uses checked manifest discovery; malformed Gradle settings exit 1 before generation, with explicit inconclusive JSON in machine mode.

## Files to Read First

- `src/commands/generate.rs`
- `src/generator.rs`
- `src/cli.rs`

## Current Status

Stable agent-native command. Enrichment belongs to native agent skills or MCP clients.
