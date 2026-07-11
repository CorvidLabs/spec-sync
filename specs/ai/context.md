---
spec: ai.spec.md
---

## Decision

SpecSync 5.0 removed embedded inference to reduce credential exposure, automatic source transmission, provider coupling, and arbitrary shell execution. The product boundary is now deterministic scaffolding and validation plus integrations that let an already-configured coding agent enrich markdown using its own trust boundary.

## Current Status

Deprecated tombstone. There is no source file or runtime API for this module.

## Migration

Remove AI provider/model/key/URL/timeout/command settings, run `specsync generate`, and use `specsync agents install` or `specsync mcp` for agent-driven enrichment.
