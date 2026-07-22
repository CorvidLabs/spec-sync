---
change: CHG-0062-harden-mcp-root-confinement-write-authorization-argument-validation-and-notif
artifact: design
---

# Design

## Capability Model

- `run_mcp_server(root, allow_write)` owns one canonical server root.
- Read tools are always available. Their optional root is accepted only when it canonicalizes to
  the server root or an existing descendant.
- Mutating tools are advertised only when `allow_write` is true. Direct calls in read-only mode are
  rejected before execution; authorized mutators reject a per-call root and use the server root.
- Before downstream discovery, every selected project reload validates config/metadata/cache
  locations, configured path fields, manifest workspace/autodetection paths, dependency references,
  module names/files, spec mappings, and nested symlink targets. Missing write destinations validate
  through their nearest existing ancestor.
- Source/configuration tree checks canonicalize only symlinks, honor ignored or configured-excluded
  directories, and stop at deterministic entry/manifests limits. No-config autodetection preflights
  the same four-level tree that source discovery can inspect.

## Protocol Model

- Validate `params`, tool name, and arguments before dispatch.
- Tool schemas set `additionalProperties: false` and describe the same accepted arguments enforced
  at runtime.
- Protocol-shape violations return `-32602`; domain/tool failures return `isError: true`.
- A missing `id` marks a notification. Every notification, including an unknown method, emits no
  response; mutating notifications are rejected before tool execution.

## Failure Safety

Authorization failure occurs before config loading, source discovery, or writes. Unresolvable
server roots fail startup with exit code 2 and an actionable diagnostic. Config- and content-derived
escapes fail before validation, scoring, resource reads, manifest recursion, source autodetection,
generation, or initialization can reach the selected path. Tests retain outside victim files and
assert byte identity after rejected path and mutation attempts.
