---
spec: registry.spec.md
---

## Key Decisions

- **TOML registry format**: Generated registries use `[registry]` metadata plus a `[specs]` module-to-path table. The documented `[[modules]]` array-of-tables shape is accepted for compatibility.
- **GitHub raw URL fetching**: Remote registries are fetched from GitHub's raw content URL (`raw.githubusercontent.com`) with a 10-second HTTP timeout. No authentication required for public repos.
- **Checked TOML parsing**: Registry syntax is parsed with the `toml` crate. Known fields are validated for type and shape, and duplicate mappings fail closed rather than silently selecting a path.
- **Template files excluded**: Specs starting with `_` (like `_template.spec.md`) are skipped during registry generation to keep the registry clean.
- **Module name from frontmatter**: The registry extracts the `module` field from each spec's frontmatter rather than inferring from file paths, ensuring consistency with the spec's own identity.
- **Alphabetical sorting**: Generated registry entries are sorted by module name for deterministic output.

## Files to Read First

- `src/registry.rs` — Single-file module with registry generation, loading, parsing, and remote fetching.

## Current Status

Fully implemented. Local registry generation and remote fetching both work. The `resolve` CLI command uses this module for cross-project dependency validation. Both `[specs]` and `[[modules]]` mappings are retained. Inert 5.0.1-era stubs load as absent, while malformed and non-inert invalid registries fail closed.

## Notes

- Remote registry fetching is the only network operation in the default (non-AI) workflow. The `--remote` flag on `specsync resolve` triggers it.
- The cross-project reference format `owner/repo@module` is parsed by the validator module, not the registry module.
