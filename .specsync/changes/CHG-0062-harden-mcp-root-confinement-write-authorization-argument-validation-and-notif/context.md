---
change: CHG-0062-harden-mcp-root-confinement-write-authorization-argument-validation-and-notif
artifact: context
---

# Context

Issue #414 demonstrates that every MCP tool currently trusts its optional `root` argument. A client
can point `specsync_generate` or `specsync_init` at any writable host path, turning an agent prompt
injection into an arbitrary file-write primitive. The same handler also coerces wrong argument types
and replies to known notifications with `id: null`.

The current canonical MCP contract explicitly permits unrestricted root overrides and always lists
seven tools, so this is an intentional public-contract correction. The CLI argument contract must
add an operator-controlled write capability, and the root CLI dispatcher contract must bind that
capability to MCP startup while surfacing root-resolution failures as usage errors.

Security boundary: the canonical server root is the authority boundary. Read calls may inspect an
existing descendant; write calls may mutate only the server root and only after `--allow-write`.

Implementation status: `src/mcp.rs` now canonicalizes the server root once, suppresses every parsed
notification before dispatch, validates exact call and argument shapes, and separates protocol
errors from tool-domain errors. `src/cli.rs` and the canonically owned `src/main.rs` dispatcher carry
the explicit write capability.
Pre-load confinement covers configuration/metadata/cache files, manifest workspaces, source
autodetection, dependency references, module/spec paths, and write destinations with bounded,
ignore-aware symlink scans. Integration coverage preserves the shared read-only helper, adds a
write-enabled helper locally in the owned MCP test module, and asserts byte identity for victims
outside the configured root after adversarial paths.
