---
spec: mcp.spec.md
---

## Key Decisions

- MCP is a deterministic stdio JSON-RPC adapter for coding agents.
- Generate creates local templates only and rejects retired inference arguments.
- Tool errors use `isError`; protocol errors remain JSON-RPC errors.
- Agent credentials and model execution stay outside SpecSync.
- The canonical server root is the filesystem authority boundary. Read roots may name only existing canonical descendants.
- Mutation is an explicit server capability; mutators are hidden and denied unless `--allow-write` is present, and never accept per-call roots.
- Parsed notifications are discarded before dispatch so they cannot trigger filesystem work.
- Confinement is enforced again at downstream path sources: config/metadata/cache files, configured
  path fields, Cargo/package/Gradle/Python autodetection paths, dependency references, module
  names/files, spec mappings, nested symlinks, generated destinations, and init destinations.
- Recursive checks canonicalize only symlinks, honor excluded directories, and stop at deterministic
  entry/manifests bounds; no-config source autodetection is preflighted to the same four-level scope.

## Files to Read First

- `src/mcp.rs`
- `src/generator.rs`
- `src/validator.rs`

## Current Status

Stable, read-only-by-default agent-native MCP integration with exact JSON-RPC argument validation,
request-, configuration-, and autodetection-level confinement, explicit root-bound mutation, and no
embedded provider or credential surfaces.
