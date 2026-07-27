---
change: CHG-0062-harden-mcp-root-confinement-write-authorization-argument-validation-and-notif
artifact: docs
---

# Docs

- Update the MCP/CLI reference to state that the server is read-only by default.
- Document `specsync mcp --allow-write` as an explicit capability grant scoped to the configured
  root.
- Document that read-tool root overrides are confined to existing descendants and that mutating
  tools reject root overrides.
- Document pre-load confinement for metadata/cache, manifest workspace, dependency, and nested
  symlink paths plus bounded ignore-aware autodetection.
- Add a migration note for clients that currently expect `specsync_generate` or `specsync_init` in
  the default tool list.

## Completed Updates

- `site/src/content/docs/cli.md` documents the read-only default, exact tool sets, direct and
  indirect root confinement, bounded autodetection, `--allow-write`, and the migration requirement.
- `site/src/content/docs/integrations/ai-agents.md` provides read-only and write-enabled client setup
  and explains that mutators are fixed to the configured root.
- `site/src/content/docs/workflow.md` and `site/src/content/docs/why-specsync.md` no longer imply that
  default MCP mode can generate files.
