---
spec: config.spec.md
---

## Key Decisions

- **JSON-first, TOML fallback**: `specsync.json` is checked first for familiarity with the JS/TS ecosystem; `.specsync.toml` is the alternative for Rust/Go projects. Both produce the same `SpecSyncConfig`.
- **Zero-dependency TOML**: Rather than pulling in a TOML crate, parsing is done line-by-line with string operations. This keeps the dependency tree minimal and avoids version conflicts.
- **Auto-detection fallback**: If no config file exists, source directories are detected by scanning for files with recognized extensions up to 3 levels deep. Fallback is `["src"]` if nothing found.
- **Manifest-first discovery**: When detecting source dirs, manifest files (Cargo.toml, Package.swift, etc.) are checked before falling back to extension scanning. This gives more accurate module-aware results.
- **46 hardcoded excludes**: Common build/cache directories (node_modules, target, .git, dist, etc.) are always excluded to prevent scanning generated code.

## Files to Read First

- `src/config.rs` — Config loading, TOML parsing, source directory detection, and manifest integration.
- `src/types.rs` — `SpecSyncConfig` struct definition with all field defaults.

## Current Status

Fully implemented. JSON and TOML loading work with unknown-key warnings. Auto-detection covers all 9 supported languages. Manifest-aware discovery delegates to the manifest module.

## Notes

- The config module is the bridge between user intent (config file) and the rest of the system — validator, watch, and MCP all depend on it.
- Unknown keys in config files produce warnings, not errors, for forward compatibility when newer config options are added.
