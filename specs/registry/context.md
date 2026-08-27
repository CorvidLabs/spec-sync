---
spec: registry.spec.md
---

## Key Decisions

- **TOML registry format**: The registry lives at `.specsync/registry.toml`; root-level `specsync-registry.toml` survives only as the un-migrated 3.x fallback, and `local_registry_path` decides which one applies. It carries a `[registry]` section for metadata and a `[specs]` section mapping module names to spec file paths; the `[[modules]]` array-of-tables shape is also accepted.
- **GitHub raw URL fetching**: Remote registries are fetched from GitHub's raw content URL (`raw.githubusercontent.com`) with a 10-second HTTP timeout. No authentication required for public repos.
- **Real TOML parsing**: `parse_registry_toml` uses the in-process `toml` crate and returns `Err` on malformed input, so a caller fails closed instead of silently dropping mappings. Generation is still string construction, guarded by `toml_escape`.
- **Template files excluded**: Specs starting with `_` (like `_template.spec.md`) are skipped during registry generation to keep the registry clean.
- **Module name from frontmatter**: The registry extracts the `module` field from each spec's frontmatter rather than inferring from file paths, ensuring consistency with the spec's own identity.
- **Alphabetical sorting**: Generated registry entries are sorted by module name for deterministic output.

## Files to Read First

- `src/registry.rs` — Single-file module with registry generation, loading, parsing, and remote fetching.

## Current Status

Fully implemented. Local registry generation and remote fetching both work. The `resolve` CLI command uses this module for cross-project dependency validation. Inert 5.0.1-era stubs (no registry `name`, no `[specs]` mappings) load as absent through `load_local_registry` so module resolution can fall back to conventional paths.

## Notes

- Remote registry fetching is this module's only network operation, triggered by the `--remote` flag on `specsync resolve`. It is not the project's only one: `specsync issues` and batch issue verification reach `api.github.com` through `src/github.rs`.
- The cross-project reference format `owner/repo@module` is parsed by the validator module, not the registry module.
