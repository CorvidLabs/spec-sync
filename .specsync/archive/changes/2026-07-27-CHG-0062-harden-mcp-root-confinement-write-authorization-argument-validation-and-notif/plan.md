---
change: CHG-0062-harden-mcp-root-confinement-write-authorization-argument-validation-and-notif
artifact: plan
---

# Plan

1. Add `--allow-write` to the MCP CLI variant and pass it to `run_mcp_server`.
2. Canonicalize the configured root once; reject a server root that cannot be canonicalized.
3. Split tool metadata into read-only and mutating capabilities. List mutators only in write mode,
   and reject direct mutator calls before execution when write mode is disabled.
4. Resolve read roots through one confinement helper. Reject non-descendants, traversal, and
   symlink escapes. Reject all root overrides on mutating tools. Apply the same boundary to config
   files and configured paths, metadata/cache/schema files, manifest workspace and autodetection
   paths, dependency references, module names/files, spec mappings, nested symlink targets,
   generated destinations, and initialization destinations. Bound recursive preflights and honor
   ignored/configured-excluded directories.
5. Replace permissive argument access with exact object/schema validation and JSON-RPC `-32602`
   protocol errors for malformed calls.
6. Suppress responses for every notification, including unknown methods, and forbid notification
   mutations.
7. Add adversarial integration tests and update MCP, CLI-argument, and root-dispatcher canonical
   specs, companions, and docs.
8. Run targeted tests, full repository verification, independent correctness review, and an
   adversarial security review.
