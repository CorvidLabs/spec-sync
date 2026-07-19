---
change: CHG-0059-tolerate-inert-5-0-1-registry-toml-stubs-so-module-resolution-falls-back-to-defa
artifact: context
---

# Context

## Decision

Treat inert 5.0.1-era local registry stubs as absent during module resolution. Those stubs often contain
only `version = 1` and an empty `[modules]` table — they never carried module authority under 5.1.x
parsing (which requires a registry `name` and `[specs]` mappings). Failing closed on them forces
every adopter with a leftover stub through a pointless repair.

## Surfaces

1. `is_inert_legacy_registry_stub` / `load_local_registry` in `src/registry.rs`
2. `canonical_module_paths` in `src/change.rs` uses `load_local_registry` so inert → conventional fallback
3. Non-inert unparsable registries keep the exact pre-fix diagnostic

## Non-goals

- Migrating or rewriting adopter registry files
- Changing remote registry fetch behavior
- Weakening fail-closed parsing for named/mapped but invalid registries
