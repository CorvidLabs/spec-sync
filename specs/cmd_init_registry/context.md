---
spec: cmd_init_registry.spec.md
---

## Key Decisions

- The command is a thin wrapper around `registry::generate_registry`; all spec discovery and TOML rendering live in the `registry` module. This command only resolves the project name, guards against overwrite, and writes the file.
- Project name defaults to the root directory's file name (`root.file_name()`), with `"project"` as a last-resort fallback, so the registry is usable even when run from an oddly-named or root path.
- Like `init`, it refuses to overwrite an existing registry and simply reports that it already exists. The path it writes is layout-dependent: `.specsync/registry.toml` on the current layout, and the legacy root-level `specsync-registry.toml` only on an un-migrated 3.x project — `registry::local_registry_path` decides.

## Files to Read First

- `src/commands/init_registry.rs` — the whole command.
- `src/registry.rs` — `generate_registry(root, project_name, specs_dir)`, which discovers specs and renders the TOML.
- `src/config.rs` — `load_config_allowing_unloadable` (provides `specs_dir`; the tolerant variant, so an unreadable config cannot stop the registry from being scaffolded).

## Current Status

Implemented and stable. No unit tests live in this file; the command is driven end to end by `init_registry_uses_v4_path_in_migrated_project`, `init_registry_keeps_legacy_path_for_legacy_project`, and `init_registry_is_idempotent_for_v4_registry` in `tests/integration/config.rs`, and the registry-rendering logic is covered by the `registry` module's own tests.

## Notes

- Part of the command layer — orchestrates the `registry` module rather than containing domain logic.
